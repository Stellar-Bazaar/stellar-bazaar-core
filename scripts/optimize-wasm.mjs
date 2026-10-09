import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Import binaryen from frontend node_modules
const binaryenPath = path.resolve(__dirname, '..', '..', 'stellar-bazaar-frontend', 'node_modules', 'binaryen', 'index.js');
const binaryenModule = await import(`file://${binaryenPath.replace(/\\/g, '/')}`);
const binaryen = binaryenModule.default || binaryenModule;

const inputWasmPath = path.resolve(__dirname, '..', 'target', 'wasm32-unknown-unknown', 'release', 'demand_circle_registry.wasm');
const outputWasmPath = path.resolve(__dirname, '..', 'target', 'wasm32-unknown-unknown', 'release', 'demand_circle_registry.optimized.wasm');

console.log('Reading WASM from:', inputWasmPath);
const wasmBytes = fs.readFileSync(inputWasmPath);
console.log(`Original WASM size: ${wasmBytes.length} bytes`);

const mod = binaryen.readBinary(wasmBytes);
console.log(`Features before: ${mod.getFeatures()}`);

// Set strict WebAssembly MVP features required by Soroban host environment
mod.setFeatures(binaryen.Features.MVP);
console.log(`Features after: ${mod.getFeatures()}`);

// Optimize to canonicalize bytecodes and remove dead code
mod.optimize();

const optimizedBytes = Buffer.from(mod.emitBinary());
console.log(`Optimized WASM size: ${optimizedBytes.length} bytes`);

// Verify custom sections
console.log('Contains contractspecv0:', optimizedBytes.includes('contractspecv0'));
console.log('Contains contractmetav0:', optimizedBytes.includes('contractmetav0'));
console.log('Contains create_circle export:', optimizedBytes.includes('create_circle'));

fs.writeFileSync(outputWasmPath, optimizedBytes);
console.log(`Saved optimized WASM to: ${outputWasmPath}`);

// Also replace original for any tooling expecting default path
fs.writeFileSync(inputWasmPath, optimizedBytes);
console.log(`Updated original WASM with optimized build at: ${inputWasmPath}`);
