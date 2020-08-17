type ListenerFn<T> = (items: T[]) => void

export class Store<T> {
  private items: T[] = []
  listeners: Array<ListenerFn<T>> = []
  persistenceKey: string | undefined
  reviver?:
    | ((this: unknown, key: string, value: unknown) => unknown)
    | undefined

  constructor(
    persistenceKey?: string,
    reviver?:
      | ((this: unknown, key: string, value: unknown) => unknown)
      | undefined,
  ) {
    this.persistenceKey = persistenceKey
    this.reviver = reviver
    this.load()
  }

  addItem(item: T): void {
    this.items.push(item)
    this.notifyListeners()
    this.persist()
  }

  removeItem(item: T): void {
    this.items = this.items.filter(_ => _ !== item)
    this.notifyListeners()
    this.persist()
  }

  getItems(): T[] {
    return this.items
  }

  registerListener(listener: ListenerFn<T>): void {
    this.listeners.push(listener)
  }

  unregisterListener(listener: ListenerFn<T>): void {
    this.listeners = this.listeners.filter(_ => _ !== listener)
  }

  private notifyListeners() {
    this.listeners.forEach(listener => listener(this.items))
  }

  private persist() {
    if (this.persistenceKey && "localStorage" in globalThis) {
      localStorage.setItem(this.persistenceKey, JSON.stringify(this.items))
    }
  }

  private load() {
    if (this.persistenceKey && "localStorage" in globalThis) {
      try {
        const rawItems = localStorage.getItem(this.persistenceKey)
        if (rawItems) {
          const items = JSON.parse(rawItems, this.reviver)
          this.items = items
        }
      } catch (_) {
        console.info(
          `Didn't find any persisted data for ${this.persistenceKey}`,
        )
      }
    }
  }
}
