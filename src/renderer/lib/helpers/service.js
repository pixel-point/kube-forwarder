export function getServiceLabel(service) {
  return service.alias || service.workloadName
}
