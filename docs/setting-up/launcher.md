# Launcher configuration

Currently, the only supported deployment method is GitHub Actions, as it's the easiest way to build for Windows and macOS.

Start by forking the https://github.com/Petr1Furious/potato_launcher repository.

Then, you have two options to configure your launcher: config file or environment variables.

### Config file

::: info Pros and cons
- Pro: Everything needed to build the launcher is inside the repo, so you
  (or your users) can clone the repo and build the launcher right away
- Con: May be harder to update when the upstream updates
:::

To use this option, edit the `build.env` file at the root of the repository. Each line corresponds to a variable. Everything before `=` is the name, everything after is a value. Comments are not supported, but empty lines are ignored.

### Environment variables

::: info Pros and cons
- Pro: Doesn't require changing files in the repository, so it's easier to update your repo when the upstream updates
- Con: Nix package won't work out of the box
:::

To use this option, go to the "Settings" tab, then choose "Secrets and variables" on the left and select "Actions". Then, add your config variables in the "Variables" tab. Make sure to not accidentally create them as secrets instead.


## Variables list

- **LAUNCHER_NAME** (required): Your launcher name, for example "Potato Launcher". Can only contain Latin letters, numbers, spaces, `-`, `_` and `'`
- **VERSION_MANIFEST_URL** (required): URL of a version manifest. You'll probably want to set it to `https://your.domain/data/version_manifest.json`. The vanilla URL is `https://piston-meta.mojang.com/mc/game/version_manifest_v2.json`
- **LAUNCHER_APP_ID** (required): An application ID in a [reverse domain notation](https://en.wikipedia.org/wiki/Reverse_domain_name_notation). Used in macOS and Flatpak packages. For example, `me.petr1furious.PotatoLauncher`
- **LAUNCHER_ICON** (required): A path to the launcher icon, relative to the repository
  root. For example, `packaging/potato_launcher.png`
  
  If you are using the environment variables option, this can also be a URL.
- **BACKEND_API_BASE** (optional): An URL that will be used to download launcher updates. Doesn't impact instance download. Set it to `https://your.domain/api/v1` if you want the launcher to update automatically (you want to). Also used to generate the `.flatpakref` file
- **LAUNCHER_DESCRIPTION** (optional): The application description. Used in `.desktop` files in the Nix and Flatpak packages, can safely be omitted.
- **LAUNCHER_KEYWORDS** (optional): The semicolon-separated list of additional keywords for the `.desktop` file. Can safely be omitted

## Setting up secrets

You also need to set secrets for GitHub Actions to copy the launcher to your server.

Go to the "Settings" tab, then choose "Secrets and variables" on the left and select "Actions". Then, add the secrets in the "Secrets" tab.

You need to set the following secrets:
- **SSH_KEY**: Set this to the key you generated when creating the user during server configuration
- **SERVER_ADDR**: The address of your server to connect to
- **SERVER_USER**: The created user username
- **SERVER_PATH**: The path to upload the files to. Should be `<your-root-dir>/launcher`, for example `/srv/potatosmp/launcher`
- **POST_DEPLOY_SCRIPT_PATH** (optional): The path to a script to run after every deployment. For example, it can be used to clear CloudFlare cache when using it.

## Triggering the build

All done! Make sure to enable actions in your fork by going to the "Actions" tab and clicking "I understand my workflows". Then, you can trigger the build either by pushing to the master branch or by going to the "Actions" tab, selecting "Build & Deploy" on the left and clicking the "Run workflow" button.

After a successful workflow run, the launcher binaries and packages should be available at `https://your.domain/launcher`
