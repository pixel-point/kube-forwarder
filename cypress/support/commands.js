// ***********************************************
// This example commands.js shows you how to
// create various custom commands and overwrite
// existing commands.
//
// For more comprehensive examples of custom
// commands please read more here:
// https://on.cypress.io/custom-commands
// ***********************************************

import { addMatchImageSnapshotCommand } from 'cypress-image-snapshot/command'

// -- This is a parent command --
// Cypress.Commands.add("login", (email, password) => { ... })
//
//
// -- This is a child command --
// Cypress.Commands.add("drag", { prevSubject: 'element'}, (subject, options) => { ... })
//
//
// -- This is a dual command --
// Cypress.Commands.add("dismiss", { prevSubject: 'optional'}, (subject, options) => { ... })
//
//
// -- This is will overwrite an existing command --
// Cypress.Commands.overwrite("visit", (originalFn, url, options) => { ... })

addMatchImageSnapshotCommand()

Cypress.Commands.add('updateVueState', (state) => {
  cy.window().then((win) => {
    win.vue.$store.replaceState({
      ...win.vue.$store.state,
      ...state
    })
  })
})

Cypress.Commands.add('adjustViewport', (state) => {
  cy.window().then((win) => {
    const ratio = win.devicePixelRatio
    cy.log(win.devicePixelRatio)
    cy.viewport(1000 / ratio, 660 / ratio)
  })
})
