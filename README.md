# Homeward Link
<img src="app-icon.png" width=100%, height=100px />
Temporary Icon ^

`Homeward Link` lets you display telemetry data for live and past rocket flights. 


# Contents
- [About](#about) 
- [Installation](#installation)
    - [Using a pre-built installer](#method-1-using-a-pre-built-installer)
    - [Building your own installer](#method-2-building-an-installer-locally)
- [Technologies](#technologies)
- [Running Locally](#how-to-runmodify-locally)
- [License](#license)
- [Feedback](#feedback)


## About


## Technologies
Frameworks: [Tauri](https://v2.tauri.app/) + [Vite](https://vite.dev/)

Frontend: SvelteKit + Typescript

Backend: [Rust](https://rust-lang.org/tools/install/)

In order to run locally, you only need to install [Rust](https://rust-lang.org/tools/install/) and [Node Package Manager](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm)

## Installation

#### Method 1: Using a pre-built Installer
- Navigate to [The Releases Page](github.com/herbgrace/Homeward-Link/releases)
- Find the latest release (At the top)
- Download and run the correct installer for your Operating System

#### Method 2: Building an Installer Locally
Make sure you have installed npm and other required technologies
```bash
$ git clone https://www.github.com/Homeward-Link/
$ cd Homeward-Link
$ npm run tauri build
```
Your built installer can be found at `Homeward-Link\src-tauri\target\release\bundle`

## How to Run/Contribute Locally

#### Running Locally
```bash
$ git clone https://www.github.com/Homeward-Link/
$ cd Homeward-Link
$ npm run tauri dev
```

#### Contributing Locally
```bash
$ git clone https://www.github.com/Homeward-Link/
$ cd Homeward-Link
```
Once the repo is cloned, feel free to make any changes you'd like! <br/>
The `master` branch is protected, so in order to commit changes, make a new branch. <br/>
Once your feature is complete, open a Pull Request for approval

#### Recommended IDE Setup:

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

## License
`Homeward-Link` is built as a completely open-source 

## Feedback