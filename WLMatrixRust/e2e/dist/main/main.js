"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
const axios_1 = __importStar(require("axios"));
let requestBody = `<?xml version="1.0" encoding="UTF-8"?>
<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope" xmlns:wsse="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd" xmlns:saml="urn:oasis:names:tc:SAML:1.0:assertion" xmlns:wsp="http://schemas.xmlsoap.org/ws/2004/09/policy" xmlns:wsu="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd" xmlns:wsa="http://www.w3.org/2005/08/addressing" xmlns:wssc="http://schemas.xmlsoap.org/ws/2005/02/sc" xmlns:wst="http://schemas.xmlsoap.org/ws/2005/02/trust">
    <s:Header>
        <wsa:Action s:mustUnderstand="1">http://schemas.xmlsoap.org/ws/2005/02/trust/RST/Issue</wsa:Action>
        <wsa:To s:mustUnderstand="1">HTTPS://127.0.0.1:80//RST2.srf</wsa:To>
        <wsa:MessageID>1650180844</wsa:MessageID>
        <ps:AuthInfo xmlns:ps="http://schemas.microsoft.com/Passport/SoapServices/PPCRL" Id="PPAuthInfo">
            <ps:HostingApp>{7108E71A-9926-4FCB-BCC9-9A9D3F32E423}</ps:HostingApp>
            <ps:BinaryVersion>5</ps:BinaryVersion>
            <ps:UIVersion>1</ps:UIVersion>
            <ps:Cookies></ps:Cookies>
            <ps:RequestParams>AQAAAAIAAABsYwQAAAAyMDYw</ps:RequestParams>
        </ps:AuthInfo>
        <wsse:Security>
            <wsse:UsernameToken wsu:Id="user">
                <wsse:Username>aeoncl@shlasouf.internal</wsse:Username>
                <wsse:Password>passwd</wsse:Password>
            </wsse:UsernameToken>
            <wsu:Timestamp Id="Timestamp">
                <wsu:Created>2022-04-17T09:34:04Z</wsu:Created>
                <wsu:Expires>2022-04-17T09:39:04Z</wsu:Expires>
            </wsu:Timestamp>
        </wsse:Security>
    </s:Header>
    <s:Body>
        <ps:RequestMultipleSecurityTokens xmlns:ps="http://schemas.microsoft.com/Passport/SoapServices/PPCRL" Id="RSTS">
            <wst:RequestSecurityToken Id="RST0">
                <wst:RequestType>http://schemas.xmlsoap.org/ws/2005/02/trust/Issue</wst:RequestType>
                <wsp:AppliesTo>
                    <wsa:EndpointReference>
                        <wsa:Address>http://Passport.NET/tb</wsa:Address>
                    </wsa:EndpointReference>
                </wsp:AppliesTo>
            </wst:RequestSecurityToken>
            <wst:RequestSecurityToken Id="RST1">
                <wst:RequestType>http://schemas.xmlsoap.org/ws/2005/02/trust/Issue</wst:RequestType>
                <wsp:AppliesTo>
                    <wsa:EndpointReference>
                        <wsa:Address>messengerclear.live.com</wsa:Address>
                    </wsa:EndpointReference>
                </wsp:AppliesTo>
                <wsp:PolicyReference URI="MBI_KEY_OLD"></wsp:PolicyReference>
            </wst:RequestSecurityToken>
            <wst:RequestSecurityToken Id="RST2">
                <wst:RequestType>http://schemas.xmlsoap.org/ws/2005/02/trust/Issue</wst:RequestType>
                <wsp:AppliesTo>
                    <wsa:EndpointReference>
                        <wsa:Address>messenger.msn.com</wsa:Address>
                    </wsa:EndpointReference>
                </wsp:AppliesTo>
                <wsp:PolicyReference URI="?id=507"></wsp:PolicyReference>
            </wst:RequestSecurityToken>
            <wst:RequestSecurityToken Id="RST3">
                <wst:RequestType>http://schemas.xmlsoap.org/ws/2005/02/trust/Issue</wst:RequestType>
                <wsp:AppliesTo>
                    <wsa:EndpointReference>
                        <wsa:Address>messengersecure.live.com</wsa:Address>
                    </wsa:EndpointReference>
                </wsp:AppliesTo>
                <wsp:PolicyReference URI="MBI_SSL"></wsp:PolicyReference>
            </wst:RequestSecurityToken>
            <wst:RequestSecurityToken Id="RST4">
                <wst:RequestType>http://schemas.xmlsoap.org/ws/2005/02/trust/Issue</wst:RequestType>
                <wsp:AppliesTo>
                    <wsa:EndpointReference>
                        <wsa:Address>contacts.msn.com</wsa:Address>
                    </wsa:EndpointReference>
                </wsp:AppliesTo>
                <wsp:PolicyReference URI="MBI"></wsp:PolicyReference>
            </wst:RequestSecurityToken>
            <wst:RequestSecurityToken Id="RST5">
                <wst:RequestType>http://schemas.xmlsoap.org/ws/2005/02/trust/Issue</wst:RequestType>
                <wsp:AppliesTo>
                    <wsa:EndpointReference>
                        <wsa:Address>storage.msn.com</wsa:Address>
                    </wsa:EndpointReference>
                </wsp:AppliesTo>
                <wsp:PolicyReference URI="MBI"></wsp:PolicyReference>
            </wst:RequestSecurityToken>
            <wst:RequestSecurityToken Id="RST6">
                <wst:RequestType>http://schemas.xmlsoap.org/ws/2005/02/trust/Issue</wst:RequestType>
                <wsp:AppliesTo>
                    <wsa:EndpointReference>
                        <wsa:Address>sup.live.com</wsa:Address>
                    </wsa:EndpointReference>
                </wsp:AppliesTo>
                <wsp:PolicyReference URI="MBI"></wsp:PolicyReference>
            </wst:RequestSecurityToken>
        </ps:RequestMultipleSecurityTokens>
    </s:Body>
</s:Envelope>`;
axios_1.default.post("http://127.0.0.1:8080/RST2.srf", requestBody, { headers: new axios_1.AxiosHeaders().set("Content-Type", "application/soap+xml") }).then((result) => {
    console.log(result);
}, (error) => {
    console.log(error);
});
