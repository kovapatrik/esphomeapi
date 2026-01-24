import { initLogger, discover } from '@kovapatrik/esphomeapi-manager';

initLogger(console);

const devices = await discover(2);
console.log("number of devices found:", devices.length);
