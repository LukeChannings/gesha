import {
  mount,
  MountableComponent,
  MountAttributeNotFoundError,
  ComponentNotDefinedError,
} from "./mount"

describe("mount", () => {
  it("throws a mountAttributeNotFoundError when given an element without a data-mount attr", () => {
    const div = document.createElement("div")
    expect(() => mount(div, {})).toThrow(MountAttributeNotFoundError)
  })

  it("throws a ComponentNotDefinedError when ", () => {
    const div = document.createElement("div")
    div.setAttribute("data-mount", "NotReal")
    expect(() => mount(div, {})).toThrow(ComponentNotDefinedError)
  })

  it("constructs and returns a class", () => {
    const div = document.createElement("div")
    div.setAttribute("data-mount", "MyComponent")
    expect(mount(div, { MyComponent: MountableComponent })).toBeInstanceOf(
      MountableComponent,
    )
  })
})

describe("MountableComponent", () => {
  it("exposes a node property on the instance", () => {
    const node = document.createElement("div")
    const mountable = new MountableComponent(node)
    expect(mountable.node).toBe(node)
  })
})
