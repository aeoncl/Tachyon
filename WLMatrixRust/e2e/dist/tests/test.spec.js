"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const net_1 = __importDefault(require("net"));
const promise_socket_1 = __importDefault(require("promise-socket"));
describe('Happy path handshake tests', () => {
    let client;
    let pClient;
    beforeAll(() => __awaiter(void 0, void 0, void 0, function* () {
        const port = 1863;
        const host = '127.0.0.1';
        // Create a new TCP client.
        client = new net_1.default.Socket();
        pClient = new promise_socket_1.default(client);
        // Send a connection request to the server.
        let connectResult = yield pClient.connect({ port: port, host: host });
    }));
    afterAll(() => __awaiter(void 0, void 0, void 0, function* () { return yield pClient.end(); }));
    // afterEach(() => client.removeAllListeners("data"))
    it('VER Negociation test', () => __awaiter(void 0, void 0, void 0, function* () {
        let written = yield pClient.writeAll("VER 1 MSNP18 MSNP17 CVR0\r\n");
        let read = yield pClient.read();
        expect(read === null || read === void 0 ? void 0 : read.toString()).toBe("VER 1 MSNP18\r\n");
    }));
    it("CVR Negociation test", () => __awaiter(void 0, void 0, void 0, function* () {
        let written = yield pClient.writeAll("CVR 2 0x0409 winnt 6.0.0 i386 MSNMSGR 14.0.8117.0416 msmsgs aeoncl@shlasouf.internal\r\n");
        let read = yield pClient.read();
        expect(read === null || read === void 0 ? void 0 : read.toString()).toBe("CVR 2 14.0.8117.0416 14.0.8117.0416 14.0.8117.0416 localhost localhost\r\n");
    }));
    it("USR I Phase", () => __awaiter(void 0, void 0, void 0, function* () {
        let written = yield pClient.writeAll("USR 3 SSO I aeoncl@shlasouf.internal\r\n");
        let read = yield pClient.read();
        expect(read === null || read === void 0 ? void 0 : read.toString()).toContain("USR 3 SSO S MBI_KEY_OLD");
    }));
    it("USR S Phase", () => __awaiter(void 0, void 0, void 0, function* () {
        let written = yield pClient.writeAll("USR 4 SSO S t=syt_myam4zingt0ken gaeraefaefaefae {55192CF5-588E-4ABE-9CDF-395B616ED85B}\r\n");
        let read = yield pClient.read();
        expect(read === null || read === void 0 ? void 0 : read.toString()).toContain("USR 4 OK aeoncl@shlasouf.internal 1 0");
        let read2 = yield pClient.read();
        expect(read2 === null || read2 === void 0 ? void 0 : read2.toString()).toContain("Hotmail");
    }));
});
