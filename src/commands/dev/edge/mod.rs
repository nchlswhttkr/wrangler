mod server;
mod setup;

use server::serve;

use crate::commands;
use crate::commands::dev::{socket, ServerConfig};
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::{DeployConfig, Target};

use tokio::runtime::Runtime as TokioRuntime;

pub fn dev(
    target: Target,
    deploy_config: DeployConfig,
    user: GlobalUser,
    server_config: ServerConfig,
) -> Result<(), failure::Error> {
    commands::build(&target)?;
    let init = setup::init(&deploy_config, &user)?;
    let mut target = target.clone();
    // TODO: replace asset manifest parameter
    let preview_token =
        setup::upload(&mut target, None, &deploy_config, &user, init.preview_token)?;
    // TODO: ws://{your_zone}/cdn-cgi/workers/preview/inspector
    // also need to send init.ws_token as cf-workers-preview-token on init
    let socket_url = format!(
        "wss://rawhttp.cloudflareworkers.com/inspect/{}",
        init.ws_token
    );
    let socket_url = Url::parse(&socket_url)?;
    let devtools_listener = socket::listen(socket_url);

    let server = serve(server_config, preview_token, host);

    let runners = futures::future::join(devtools_listener, server);

    let mut runtime = TokioRuntime::new()?;
    runtime.block_on(async {
        let (devtools_listener, server) = runners.await;
        devtools_listener?;
        server
    })
}
