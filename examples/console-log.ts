import { initLogger, discover } from '@kovapatrik/esphomeapi-manager';

initLogger(console);

const devices = await discover(5);
console.log("number of devices found:", devices.length);
