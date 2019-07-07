import { StateBuilder } from '../../factories/state'

context('components/shared/SearchInput', () => {
  beforeEach(() => {
    const stateBuilder = new StateBuilder()
    stateBuilder.createCluster()
    cy.visit(`#`)
    cy.updateVueState(stateBuilder.getState())

    cy.get('.search-input').first().as('searchInput')
  })

  context('visual', () => {
    it('plain', () => {
      cy.get('@searchInput').matchImageSnapshot()
    })

    // TODO stabilize blinking cursor
    // it('focus', () => {
    //   cy.get('@searchInput').find('input').focus().matchImageSnapshot()
    // })

    // TODO stabilize blinking cursor
    // it('filled', () => {
    //   cy.get('@searchInput').find('input').type('query').wait(500).matchImageSnapshot()
    // })
  })
})
