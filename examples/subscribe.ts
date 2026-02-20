import { initLogger, type Light, Manager } from "@kovapatrik/esphomeapi-manager"

initLogger(console)

const manager = await Manager.connect({
  address: process.env.ESPHOME_ADDRESS ?? "address",
  port: 6053,
  psk: process.env.ESPHOME_PSK ?? "psk"
})

const entities = manager.getEntities()
console.log(entities)

const light = entities[0] as Light

light.onStateChange(state => {
  console.log("Changed state:", state)
})

process.stdin.resume()
process.on('SIGINT', () => process.exit(0))
process.on('SIGTERM', () => process.exit(0))
