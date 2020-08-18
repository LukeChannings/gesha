export class ComponentNotDefinedError extends Error {}
export class MountAttributeNotFoundError extends Error {}

export class MountableComponent {
  node: HTMLElement

  constructor(node: HTMLElement) {
    this.node = node
  }
}

const instances: MountableComponent[] = []

// mount - constructs a MountableComponent with a given HTMLElement
export const mount = (
  node: HTMLElement,
  components: Record<string, typeof MountableComponent | undefined>,
): MountableComponent => {
  const { mount } = node.dataset
  if (typeof mount !== "string") throw new MountAttributeNotFoundError()

  const ScreenClass = components[mount]

  if (!ScreenClass) {
    throw new ComponentNotDefinedError(`No class found for ${mount}`)
  }

  const instance = new ScreenClass(node)

  instances.push(instance)

  return instance
}

// getInstances will return any constructed MountableComponents for a given subclass
// This is the main way that data is shared between Screens.
// See BrewScreen#showTimerModal for an example.
export const getInstances = <T extends typeof MountableComponent>(
  ClassCtor: T,
): Array<InstanceType<T>> =>
  (instances as Array<InstanceType<T>>).filter(
    instance => instance instanceof ClassCtor,
  )
