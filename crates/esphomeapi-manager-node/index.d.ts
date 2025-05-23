/* auto-generated by NAPI-RS */
/* eslint-disable */
export declare class Manager {
  static connect(options: ConnectionOptions): Promise<Manager>
  getDeviceName(): string
  getDeviceMac(): string
  getSwitches(): Array<Switch>
}

export declare class Switch {
  get key(): number
  get name(): string
  isOn(): boolean
  turnOn(): Promise<void>
  turnOff(): Promise<void>
  toggle(): Promise<void>
}

export interface ConnectionOptions {
  address: string
  port: number
  password?: string
  expectedName?: string
  psk?: string
  clientInfo?: string
  keepAliveDuration?: number
}

export declare function discover(seconds: number): Promise<Array<ServiceInfo>>

export interface EntityInfo {
  key: number
  name: string
  uniqueId: string
  objectId: string
  deviceClass: string
  disabledByDefault: boolean
  entityCategory: string
  icon: string
}

export interface ServiceInfo {
  tyDomain: string
  subDomain?: string
  fullname: string
  server: string
  addresses: Array<string>
  port: number
  hostTtl: number
  otherTtl: number
  priority: number
  weight: number
}
