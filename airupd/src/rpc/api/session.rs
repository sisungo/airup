//! Session management APIs.

use crate::app::airupd;
use airup_sdk::rpc::{Request, Response};

async fn send_error(
    session: &mut crate::rpc::Session,
    error: airup_sdk::Error,
) -> anyhow::Result<()> {
    session
        .conn
        .send(&Response::new(Err::<(), _>(error)))
        .await?;
    Ok(())
}

pub async fn invoke(mut session: crate::rpc::Session, req: Request) {
    _ = match &req.method[..] {
        "session.into_extension" => into_extension(session, req),
        _ => send_error(&mut session, airup_sdk::Error::NotImplemented).await,
    };
}

fn into_extension(session: crate::rpc::Session, req: Request) -> anyhow::Result<()> {
    let name: String = req.extract_params()?;
    let conn = session.conn.into_inner().into_inner();
    airupd().extensions.register(name, conn)?;
    Ok(())
}
