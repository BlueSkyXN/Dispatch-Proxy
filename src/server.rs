use std::{
    fmt::Debug,
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use color_eyre::owo_colors::OwoColorize;
use eyre::Result;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpListener, TcpStream},
    sync::Semaphore,
};
use tracing::instrument;

use crate::{
    dispatcher::{Dispatch, WeightedAddress, WeightedRoundRobinDispatcher},
    socks::SocksHandshake,
};

#[instrument]
async fn handle_socket<D>(mut socket: TcpStream, dispatcher: D) -> Result<()>
where
    D: Dispatch + Debug,
{
    let mut server_socket = {
        let (client_reader, client_writer) = socket.split();

        let mut handshake = SocksHandshake::new(client_reader, client_writer, dispatcher);

        match handshake.handshake().await {
            Err(err) => {
                return Err(err.wrap_err(eyre::eyre!(
                    "An error occurred during the proxy handshake procedure"
                )));
            }
            Ok(server_socket) => server_socket,
        }
    };

    let local_addr = match socket.peer_addr() {
        Ok(local_addr) => local_addr,
        Err(err) => match err.kind() {
            // InvalidInput: Invalid argument
            // Occurs if the socket was closed in the time it took us to get to this point.
            std::io::ErrorKind::InvalidInput => return Ok(()),
            _ => return Err(err.into()),
        },
    };
    let remote_addr = server_socket.peer_addr()?;
    tracing::info!(
        "connection initiated between {} and {}",
        local_addr,
        remote_addr
    );

    let (client_reader, client_writer) = socket.split();
    let (server_reader, server_writer) = server_socket.split();

    // TODO: we can get a connection reset by peer here.
    pipe_multiple(client_reader, client_writer, server_reader, server_writer).await?;

    tracing::info!(
        "connection terminated between {} and {}",
        local_addr,
        remote_addr
    );

    Ok(())
}

async fn pipe<R, W>(mut reader: R, mut writer: W) -> Result<()>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    if let Err(err) = tokio::io::copy(&mut reader, &mut writer).await {
        match err.kind() {
            // Connection reset by peer
            // We currently don't have a way to propagate this error in either direction, so instead we act as if
            // the stream ended gracefully (EOF).
            std::io::ErrorKind::ConnectionReset => Ok(()),
            _ => Err(eyre::eyre!(err)),
        }
    } else {
        Ok(())
    }
}

async fn pipe_multiple<R1, W1, R2, W2>(
    reader1: R1,
    writer1: W1,
    reader2: R2,
    writer2: W2,
) -> Result<()>
where
    R1: AsyncRead + Unpin,
    W1: AsyncWrite + Unpin,
    R2: AsyncRead + Unpin,
    W2: AsyncWrite + Unpin,
{
    let pipe1 = pipe(reader1, writer2);
    let pipe2 = pipe(reader2, writer1);

    tokio::pin!(pipe1, pipe2);

    tokio::select! {
        res = pipe1 => res,
        res = pipe2 => res
    }
}

#[instrument]
async fn start_server(addr: SocketAddr, addresses: Vec<WeightedAddress>) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;

    // Security warning for non-localhost binding
    if !addr.ip().is_loopback() {
        use owo_colors::OwoColorize;
        eprintln!(
            "{} {}",
            "⚠ WARNING:".yellow().bold(),
            format!(
                "The proxy is bound to {} which is accessible from other machines on your network.",
                addr.ip()
            )
            .yellow()
        );
        eprintln!(
            "{}",
            "  This proxy does not require authentication and anyone who can reach this address can use it."
                .yellow()
        );
        eprintln!(
            "{}",
            "  Consider binding to 127.0.0.1 (localhost) unless you specifically need external access.\n"
                .yellow()
        );
    }

    println!("SOCKS proxy started on {}", addr.bold());
    println!(
        "Dispatching to {} {}",
        if addresses.len() > 1 {
            "addresses"
        } else {
            "address"
        },
        addresses
            .iter()
            .map(|addr| format!("{}", addr.bold()))
            .collect::<Vec<_>>()
            .join(",")
    );

    let dispatcher = WeightedRoundRobinDispatcher::new(addresses);
    
    // Connection limiting: Allow up to 1000 concurrent connections to prevent DoS
    // This is a reasonable default that can handle many simultaneous connections
    // while preventing resource exhaustion
    let connection_semaphore = Arc::new(Semaphore::new(1000));

    loop {
        let (socket, _) = listener.accept().await?;
        let dispatcher = dispatcher.clone();
        
        // Try to acquire a connection permit. If this fails (e.g., semaphore closed), 
        // we skip this connection rather than panicking.
        let permit = match connection_semaphore.clone().acquire_owned().await {
            Ok(permit) => permit,
            Err(_) => {
                tracing::error!("Failed to acquire connection permit, semaphore may be closed");
                continue;
            }
        };
        
        tokio::spawn(async move {
            // The permit will be automatically released when this task completes
            let _permit = permit;
            if let Err(err) = handle_socket(socket, dispatcher).await {
                // Errors that happen during the handling of a socket are only reported as warnings, since they're
                // considered to be recoverable. On the other hand, panics are unrecoverable and are reported as errors.
                tracing::warn!("{:?}", err);
            }
        });
    }
}

#[instrument]
pub fn server(ip: IpAddr, port: u16, addresses: Vec<WeightedAddress>) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(start_server(SocketAddr::new(ip, port), addresses))
}
