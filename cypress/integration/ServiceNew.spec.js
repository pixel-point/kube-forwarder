import { StateBuilder } from '../factories/state'

context('pages/ServiceNew', () => {
  beforeEach(() => {
    const stateBuilder = new StateBuilder()
    const cluster = stateBuilder.createCluster()

    cy.visit(`#/clusters/${cluster.id}/services/new`)
    cy.updateVueState(stateBuilder.getState())
  })

  it('success', () => {
    cy.matchImageSnapshot()
  })
})
