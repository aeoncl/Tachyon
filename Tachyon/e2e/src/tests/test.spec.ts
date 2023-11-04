import Net from "net";
import PromiseSocket from "promise-socket";

describe('Happy path handshake tests', () => {
    let client: Net.Socket;
    let pClient: PromiseSocket<Net.Socket>;

    beforeAll(async() => {
        const port = 1863;
        const host = '127.0.0.1';
        // Create a new TCP client.
        client = new Net.Socket();
        pClient = new PromiseSocket(client);
        
        // Send a connection request to the server.
        let connectResult = await pClient.connect({ port: port, host: host });
    });

    afterAll(async() => await pClient.end());


   // afterEach(() => client.removeAllListeners("data"))

    it('VER Negociation test', async() => {

        let written = await pClient.writeAll("VER 1 MSNP18 MSNP17 CVR0\r\n");

        let read = await pClient.read();

        expect(read?.toString()).toBe("VER 1 MSNP18\r\n");

    });

    it("CVR Negociation test", async() => {

        let written = await pClient.writeAll("CVR 2 0x0409 winnt 6.0.0 i386 MSNMSGR 14.0.8117.0416 msmsgs aeoncl@shlasouf.internal\r\n");

        let read = await pClient.read();

        expect(read?.toString()).toBe("CVR 2 14.0.8117.0416 14.0.8117.0416 14.0.8117.0416 localhost localhost\r\n");
    });

    it("USR I Phase", async() => {


        let written = await pClient.writeAll("USR 3 SSO I aeoncl@shlasouf.internal\r\n");

        let read = await pClient.read();
        
        expect(read?.toString()).toContain("USR 3 SSO S MBI_KEY_OLD");

    });

    it("USR S Phase", async() => {


        let written = await pClient.writeAll("USR 4 SSO S t=syt_myam4zingt0ken gaeraefaefaefae {55192CF5-588E-4ABE-9CDF-395B616ED85B}\r\n");

        let read = await pClient.read();
        
        expect(read?.toString()).toContain("USR 4 OK aeoncl@shlasouf.internal 1 0");

        let read2 = await pClient.read();
        expect(read2?.toString()).toContain("Hotmail");
    });
});
