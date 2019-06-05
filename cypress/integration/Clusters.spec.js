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

    context('dropdown visual', () => {
      beforeEach(() => {
        cy.get('.clusters > .header > .dropdown').first().as('dropdown')
      })

      it('plain', () => {
        cy.get('@dropdown').matchImageSnapshot()
      })

      // TODO
      // it('hovered', () => {
      //   cy.get('@dropdown').find('.button').trigger('mouseover').matchImageSnapshot()
      // })

      it('opened', () => {
        cy.get('@dropdown').find('.button').click().then(() => {
          cy.get('@dropdown').matchImageSnapshot()
        })
      })
    })
  })
})
