import { initLogger, discover } from '@kovapatrik/esphomeapi-manager';


initLogger({
  log: function (...data: any[]): void {
  },
  warn: function (...data: any[]): void {
  },
  error: function (...data: any[]): void {
  },
  info: console.info,
  debug: function (...data: any[]): void {
  },
  trace: function (...data: any[]): void {
  }
});

const devices = await discover(2);
console.log("number of devices found:", devices.length);
