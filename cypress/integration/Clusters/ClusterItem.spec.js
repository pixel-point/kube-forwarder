import { StateBuilder } from '../../factories/state'

context('components/shared/ClusterItem', () => {
  beforeEach(() => {
    const stateBuilder = new StateBuilder()
    stateBuilder.createCluster()
    cy.visit(`#`)
    cy.updateVueState(stateBuilder.getState())

    cy.get('.cluster-item').first().as('clusterItem')
  })

  context('visual', () => {
    it('plain', () => {
      cy.get('@clusterItem').matchImageSnapshot()
    })

    it('dropdown-opened', () => {
      cy.get('@clusterItem').find('.dropdown svg.icon_dotes').click().then(() => {
        cy.get('@clusterItem').matchImageSnapshot()
      })
    })
  })
})
