import { StateBuilder } from '../factories/state'

context('pages/ServiceClone', () => {
  beforeEach(() => {
    const stateBuilder = new StateBuilder()
    const service = stateBuilder.createService()

    cy.visit(`#`)
    cy.updateVueState(stateBuilder.getState())
    cy.visit(`#/clusters/${service.clusterId}/services/${service.id}/clone`)
  })

  it('success', () => {
    cy.matchImageSnapshot()
  })
})
