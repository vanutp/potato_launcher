<p align="center">
  <img src="packaging/potato_launcher.png" alt="Potato Launcher logo" width="200"/>
</p>

# Potato Launcher

A Minecraft launcher that is easy to use and easy to deploy with basic DevOps skills. Perfect for frequently changing modpacks.

<p align="center">
  <img src="https://github.com/user-attachments/assets/7b6f310f-c564-4d26-93cc-b709f272efdb" alt="launcher" width="400"/>
</p>

## Key Features

- **Simple and minimal**: Does all the hard work in the background, including syncing instance files, installing Java, and self-updating
- **Performant**: Downloads instances as fast as possible using an adaptive algorithm. Also, it's written in Rust, so *blazinglyfast* 🚀🚀🚀
- **Functional**: Comes with [a tool](https://launcher-docs.petr1furious.me/creating-instances) to easily create and deploy different versions and instances. The launcher can sync arbitrary files, including mods and configs, with or without overwriting them if they exist
- **Custom auth**: Supports custom authentication servers
- **Custom instances**: Players can create custom new instances with Fabric, Forge, NeoForge, or without a modloader for all Minecraft versions
- **Vanilla format**: Fully compatible with vanilla, Forge, Fabric, and NeoForge version metadata formats. It can even be built with [vanilla manifest](https://piston-meta.mojang.com/mc/game/version_manifest_v2.json) and launch vanilla versions out of the box, just like the vanilla launcher
