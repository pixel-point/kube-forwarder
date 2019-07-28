<p align="center">
  <a href="https://nodejs.org/en/">
    <img src="https://img.shields.io/badge/Node-v10.16.0-brightgreen.svg" alt="node version">
  </a>
  <a href="https://electronjs.org/">
    <img src="https://img.shields.io/badge/Electron-v5.0.6-brightgreen.svg" alt="electron version">
  </a>
  <a href="https://electronjs.org/">
    <img src="https://img.shields.io/badge/Vue-v2.6.10-brightgreen.svg" alt="vue version">
  </a>
  <a href="https://drone.pixelpoint.io/pixel-point/kube-forwarder">
    <img src="https://drone.pixelpoint.io/api/badges/pixel-point/kube-forwarder/status.svg" alt="build status">
  </a>
</p>

<h1 align="center"> Kube Forwarder</h1>
<h3 align="center">Easy to use Kubernetes port forwarding manager</h3>
<h4 align="center">
  <a href="https://kube-forwarder.pixelpoint.io">Visit a Website</a> · 
  <a href="https://github.com/pixel-point/kube-forwarder/releases/latest/download/kube-forwarder.dmg">Download for macOS</a> · 
  <a href="https://github.com/pixel-point/kube-forwarder/releases/latest/download/kube-forwarder.exe">Download for Windows</a> · 
  <a href="https://github.com/pixel-point/kube-forwarder/releases/latest/download/kube-forwarder.zip">Download for Linux</a>
  ·
  <a href="#install-with-homebrew">Install with Homebrew</a>
</h4>


![Clusters Page](https://user-images.githubusercontent.com/2697570/60103100-524d5d80-975f-11e9-84ab-bcc962be0bb7.png) 

## Features

**Auto-reconnect**

Kube Forwarder watches for connection status and always tries reconnect on failure

**Multiple clusters support**

Bookmark and forward Kubernetes services from multiple clusters easily like never before

**Share bookmarks**

Use import and export functionality to share bookmarked services with your team or simply backup it

**Zero native dependencies**

Use port-forwarding without installing kubectl and avoid explanations to developers on how to use it

## Optional dependencies

* [Google Cloud SDK](https://cloud.google.com/sdk/)
* [AWS CLI](https://docs.aws.amazon.com/cli/)

## Functionality walk-through

### Add a cluster(s)

Before you start forwarding interal resources to your local machine, you have to add cluster configuration. To do this we have 3 different options in the app:

- Auto-detection of ~/.kube/config file and parsing settings from it
- Manual adding of Kubernetes config
- Import of the JSON file that could be generated via Kube Forwarder export functionality

When you add a new cluster via auto-detection or manually, we could parse config and if there are multiple contexts inside we will suggest you to add multiple clusters to the app. Few examples of yaml files we expect to have you could find [there](https://github.com/pixel-point/kube-forwarder/issues/7)

<a target="_blank" href="https://user-images.githubusercontent.com/2697570/60754775-58a4ca80-9fe6-11e9-8d67-d15a1423b506.png"><img width="320" alt="Screenshot 2019-07-06 at 12 04 45" src="https://user-images.githubusercontent.com/2697570/60754775-58a4ca80-9fe6-11e9-8d67-d15a1423b506.png"></a>

### Add a resource

Kube Forwarder supports forwarding of all types of resources that supported by `kubectl` – Pod, Deployment, Service. 

We ask you to fill the form with the following fields:

**Cluster Name** - pick a cluster from one of the added clusters.

**Namespace** - the namespace of the resource you plan to forward.

**Kind** – pick one of the options Pod, Deployment or Service.

**Name** - name of the Pod, Deployment or Service.

**Alias** - alternative name of the resource that will be displayed on the homepage(optional)

**Port Forwarding** - Fill two fields. Local port - port from your local machine where the resource will be forwarded. Resource port - port of the resource from the Kubernetes cluster 

<a target="_blank" href="https://user-images.githubusercontent.com/2697570/60754738-e207cd00-9fe5-11e9-95b3-8f4704ca3dce.png"><img width="320" alt="Port Forwarding Form" src="https://user-images.githubusercontent.com/2697570/60754738-e207cd00-9fe5-11e9-95b3-8f4704ca3dce.png"></a>

### Import/Export 

Kube Forwarder allows you export cluster configuration in JSON that you could use to share with your team members or for the backup purpose. You could easily store it on Github. When you export cluster, you could export it with or without confidential information.

<a target="_blank" href="https://user-images.githubusercontent.com/2697570/60754844-54c57800-9fe7-11e9-9de0-fe77fc6b4290.png"><img width="320" alt="Screenshot 2019-07-06 at 12 12 20" src="https://user-images.githubusercontent.com/2697570/60754844-54c57800-9fe7-11e9-9de0-fe77fc6b4290.png"></a>

### Install with Homebrew

```
brew cask install kube-forwarder
```

## Contributing

We encourage you to contribute to Kube Forwarder!

We expect contributors to abide by our underlying [code of conduct](./.github/CODE_OF_CONDUCT.md). 
All conversations and discussions on GitHub (issues, pull requests) 
must be respectful and harassment-free.

This project was generated with [electron-vue](https://github.com/SimulatedGREG/electron-vue)@[8fae476](https://github.com/SimulatedGREG/electron-vue/tree/8fae4763e9d225d3691b627e83b9e09b56f6c935) using [vue-cli](https://github.com/vuejs/vue-cli). Documentation about the original structure can be found [here](https://simulatedgreg.gitbooks.io/electron-vue/content/index.html).

### How to contribute

1.  Fork the project & clone locally. Follow the initial setup [here](#getting-started).
2.  Create a branch, naming it either a feature or bug: `git checkout -b feature/that-new-feature` or `bug/fixing-that-bug`
3.  Code and commit your changes. Bonus points if you write a [good commit message](https://chris.beams.io/posts/git-commit/): `git commit -m 'Add some feature'`
4.  Push to the branch: `git push origin feature/that-new-feature`
5.  Create a pull request for your branch 🎉

## Getting started

### Prerequisites

* Node 10.8+
* MacOS (if you want to build `.dmg` target)
* Docker (if you want to run tests)
* ImageMagick (to build app icon)

### Installing

Fork Kube Forwarder repository (https://github.com/pixel-point/kube-forwarder/fork)

```
# Clone source code
git clone https://github.com/<your-username>/kube-forwarder

# install dependencies
npm install

# prepare .env files
cp .env.example .env
cp .env.example .env.production

# serve with hot reload in Electron Dev app
npm run dev

# serve WEB version with hot reload at localhost:9081
npm run web
```

### Build

Build an application for production
```
# Build a target for current OS
npm run build

# Build a target for Windows
npm run build -- -- --win

# Build a target for Linux
npm run build -- -- --linux

# You can mix targets
npm run build -- -- --win --linux

# You can build static and target separately
npm run build:dist
npm run build:target -- --win
```

A built version will be appear in `build` directory. 

## Running the tests

We are using [Cypress](https://www.cypress.io) to run integration tests.
There are visual regression tests. It's important to run them inside docker
container to get same screenshots as in Drone CI.

```
npm run test:cypress
```

Or you can run it manually on a local machine.
```
# Run the web version to test it
npm run web

# Run this command in a separate terminal tab
npm run test:cypress:onhost

# Or you can open Cypress GUI
npm run test:cypress:open
```

## Release guide

1) Be sure that you created `.env.production` and filled it in the right way.
2) Update the version in `package.json`.
3) Push to `release` branch. [Drone(pixel-point/kube-forwarder)](https://drone.pixelpoint.io/pixel-point/kube-forwarder/)
will build packages for Windows and Linux and upload them to [releases](https://github.com/pixel-point/kube-forwarder/releases)
4) Run `npm run release` on a Mac computer to build `.dmg` target. 
It will be automatically pushed to releases at Github.
5) Go to [Releases](https://github.com/pixel-point/kube-forwarder/releases) in the repository. 
Make sure that the created draft is OK and release it (Edit -> Release).
6) Run `cask-repair kube-forwarder` to update the cask version.
([https://github.com/Homebrew/homebrew-cask/blob/master/CONTRIBUTING.md#updating-a-cask](About cask-repair))  

Notes: 
1) `.dmg` target is added to release by your mac computer. 
`.AppImage` and `.exe` have to be added to the release by drone CI.
2) A release tag (for example: `v1.0.3`) will be added automatically 
by Github when you release your draft.

## Development tips

Use `tiffutil -cathidpicheck bg.png bg@2x.png -out bg.tiff` to build a tiff
background for .DMG 

## Supported by
<table>
  <tbody>
    <td valign="middle">
<a href="https://www.browserstack.com"><img width="200px" src="https://user-images.githubusercontent.com/2697570/60770117-f0361600-a0d7-11e9-8a1e-39393f4d0439.png" /></a>
    </td>
    <td valign="middle"><a href="https://sentry.io/"><img width="200px" src="https://user-images.githubusercontent.com/2697570/60827162-dd8f1000-a1af-11e9-9536-0bba6c71778c.png"/></a></td>
    </tbody>
</table>
 

## License

This project is licensed under the MIT License - see the LICENSE.md file for details
