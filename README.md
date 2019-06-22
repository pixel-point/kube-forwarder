# Kube Forwarder

<p>
  <a href="https://nodejs.org/en/">
    <img src="https://img.shields.io/badge/Node-v10.16.0-brightgreen.svg" alt="node version">
  </a>
  <a href="https://electronjs.org/">
    <img src="https://img.shields.io/badge/Electron-v5.0.2-brightgreen.svg" alt="rails version">
  </a>
  <a href="https://electronjs.org/">
    <img src="https://img.shields.io/badge/Vue-v2.6.10-brightgreen.svg" alt="rails version">
  </a>
  <a href="https://drone.pixelpoint.io/pixel-point/kube-forwarder">
    <img src="https://drone.pixelpoint.io/api/badges/pixel-point/kube-forwarder/status.svg" alt="Travis Status for thepracticaldev/dev.to">
  </a>
</p>

[Official Site](https://kube-forwarder.pixelpoint.io)

Easy to use Kubernetes port forwarding management. 
Built with [Electron](https://electronjs.org)

![Clusters Page](./.github/page-clusters.png?raw=true) 

## Contributing

We encourage you to contribute to Kube Forwarder!

We expect contributors to abide by our underlying [code of conduct](./.github/CODE_OF_CONDUCT.md). 
All conversations and discussions on GitHub (issues, pull requests) 
must be respectful and harassment-free.

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
npm run web

# In a separate terminal tab
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

Notes: 
1) `.dmg` target is added to release by your mac computer. 
`.AppImage` and `.exe` have to be added to the release by drone CI.
2) A release tag (for example: `v1.0.3`) will be added automatically 
by Github when you release your draft.

## Development Tips

Use `tiffutil -cathidpicheck bg.png bg@2x.png -out bg.tiff` to build a tiff
background for .DMG 

## License

This project is licensed under the MIT License - see the LICENSE.md file for details

---

This project was generated with [electron-vue](https://github.com/SimulatedGREG/electron-vue)@[8fae476](https://github.com/SimulatedGREG/electron-vue/tree/8fae4763e9d225d3691b627e83b9e09b56f6c935) using [vue-cli](https://github.com/vuejs/vue-cli). Documentation about the original structure can be found [here](https://simulatedgreg.gitbooks.io/electron-vue/content/index.html).
