import { HomeAssistantEventKind, initLogger, type Light, LogLevel, Manager } from "@kovapatrik/esphomeapi-manager"

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

manager.subscribeLogs(LogLevel.Debug, false, (event) => {
  console.log(event.message)
})

manager.subscribeHomeAssistantStates((event) => {
  console.log("STATE", event.entityId, event.eventType)
  if (event.eventType === HomeAssistantEventKind.StateSubscription) {
    let testData = true
    setInterval(async () => {
      console.log("State subscription interval", event.entityId, testData ? "42" : "0")
      await manager.sendHomeAssistantState(event.entityId, testData ? "42" : "0")
      testData = !testData
    }, 5000)
  }
})

process.stdin.resume()
process.on('SIGINT', () => process.exit(0))
process.on('SIGTERM', () => process.exit(0))
