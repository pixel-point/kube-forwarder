context('pages/ClusterNew', () => {
  beforeEach(() => {
    cy.visit('#/clusters/add')
  })

  it('success', () => {
    cy.matchImageSnapshot()
  })
})
