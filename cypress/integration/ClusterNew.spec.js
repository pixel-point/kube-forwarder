context('pages/ClusterNew', () => {
  beforeEach(() => {
    cy.visit('#/clusters/new')
  })

  it('success', () => {
    cy.matchImageSnapshot()
  })
})
