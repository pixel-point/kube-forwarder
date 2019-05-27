# Kube Forwarder

> A tool for managing port forwarding configs for kubernetes clusters

#### Build Setup

``` bash
# install dependencies
npm install

# serve with hot reload at localhost:9080
npm run dev

# build electron application for production
npm run build

# run unit & end-to-end tests
npm test


# lint all JS/Vue component files in `src/`
npm run lint

```

### Release guide

1) Update the version in `package.json`.
2) Push to `release` branch.
3) Run `npm run release` on a Mac computer to build `.dmg` target.
4) Go to Releases tab in the repository, test and release the created draft.

Notes: 
1) `.dmg` target is added to release by your mac computer. 
`.AppImage` and `.exe` have to be added to the release by drone CI.
2) A release tag (for example: `v1.0.3`) will be added automatically 
by Github when you release your draft. 

---

This project was generated with [electron-vue](https://github.com/SimulatedGREG/electron-vue)@[8fae476](https://github.com/SimulatedGREG/electron-vue/tree/8fae4763e9d225d3691b627e83b9e09b56f6c935) using [vue-cli](https://github.com/vuejs/vue-cli). Documentation about the original structure can be found [here](https://simulatedgreg.gitbooks.io/electron-vue/content/index.html).
