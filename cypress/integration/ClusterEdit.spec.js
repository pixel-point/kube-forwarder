import { StateBuilder } from '../factories/state'

context('pages/ClusterEdit', () => {
  beforeEach(() => {
    const stateBuilder = new StateBuilder()
    const cluster = stateBuilder.createCluster()

    cy.visit(`#`)
    cy.updateVueState(stateBuilder.getState())
    cy.visit(`#/clusters/${cluster.id}/edit`)
  })

  it('success', () => {
    cy.matchImageSnapshot()
  })
})
