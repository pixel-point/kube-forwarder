# Kube Forwarder

A tool for managing port forwarding configs for kubernetes clusters. 
Built with [Electron](https://electronjs.org) 

## Getting Started

### Prerequisites

* Node 10.8+
* MacOS (if you want to build `.dmg` target)
* Docker (if you want to run tests)

### Installing

```
# install dependencies
npm install

# serve with hot reload at localhost:9080
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

1) Update the version in `package.json`.
2) Push to `release` branch. [Drone(pixel-point/kube-forwarder)](https://drone.pixelpoint.io/pixel-point/kube-forwarder/)
will build packages for Windows and Linux and upload them to [releases](https://github.com/pixel-point/kube-forwarder/releases)
3) Run `npm run release` on a Mac computer to build `.dmg` target. 
It will be automatically pushed to releases at Github.
4) Go to [Releases](https://github.com/pixel-point/kube-forwarder/releases) in the repository. 
Make sure that the created draft is OK and release it (Edit -> Release). 

Notes: 
1) `.dmg` target is added to release by your mac computer. 
`.AppImage` and `.exe` have to be added to the release by drone CI.
2) A release tag (for example: `v1.0.3`) will be added automatically 
by Github when you release your draft. 

## License

This project is licensed under the MIT License - see the LICENSE.md file for details

---

This project was generated with [electron-vue](https://github.com/SimulatedGREG/electron-vue)@[8fae476](https://github.com/SimulatedGREG/electron-vue/tree/8fae4763e9d225d3691b627e83b9e09b56f6c935) using [vue-cli](https://github.com/vuejs/vue-cli). Documentation about the original structure can be found [here](https://simulatedgreg.gitbooks.io/electron-vue/content/index.html).
