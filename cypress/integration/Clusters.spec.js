import { buildState, StateBuilder } from '../factories/state'

context('pages/Clusters', () => {
  beforeEach(() => {
    cy.visit('#')
  })

  it('success', () => {
    cy.matchImageSnapshot()
  })

  context('with clusters', () => {
    beforeEach(() => {
      const stateBuilder = new StateBuilder()
      stateBuilder.createCluster()

      cy.updateVueState(stateBuilder.getState())
    })

    it('success', () => {
      cy.matchImageSnapshot()
    })
  })
})
