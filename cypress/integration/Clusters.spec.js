import { StateBuilder } from '../factories/state'

context('pages/Clusters', () => {
  beforeEach(() => {
    cy.visit('#')
  })

  context('visual', () => {
    it('plain', () => {
      cy.matchImageSnapshot()
    })
  })

  context('with clusters', () => {
    beforeEach(() => {
      const stateBuilder = new StateBuilder()
      stateBuilder.createCluster()

      cy.updateVueState(stateBuilder.getState())
    })

    context('visual', () => {
      it('plain', () => {
        cy.matchImageSnapshot()
      })
    })
  })
})
