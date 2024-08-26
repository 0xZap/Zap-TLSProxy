// need to establish mock data for the tests
// I will do after finish all the implementation

const ZapProxy = require("../src/index.js");
const net = require("net");
const tls = require("tls");
const http = require("http");

jest.mock("net");
jest.mock("tls");
jest.mock("http");

describe("ZapProxy SDK", () => {
  let zapProxy;

  beforeEach(() => {
    zapProxy = new ZapProxy("localhost", 55688, "www.example.com", 443);
    net.createConnection.mockClear();
    tls.connect.mockClear();
    http.request.mockClear();
  });

  test("should establish a connection to the proxy", async () => {
    const connectCallback = jest.fn();
    net.createConnection.mockImplementation(() => ({
      write: jest.fn(),
      on: (event, callback) => {
        if (event === "data") {
          callback("200 Connection established");
        }
        connectCallback();
      },
    }));

    await expect(zapProxy.connect()).resolves.not.toThrow();
    expect(connectCallback).toHaveBeenCalled();
  });

  test("should handle connection errors", async () => {
    net.createConnection.mockImplementation(() => ({
      on: (event, callback) => {
        if (event === "error") {
          callback(new Error("Connection error"));
        }
      },
    }));

    await expect(zapProxy.connect()).rejects.toThrow("Connection error");
  });

  test("should send a proof and receive a signature", async () => {
    const mockReq = {
      write: jest.fn(),
      end: jest.fn(),
      on: jest.fn((event, callback) => {
        if (event === "error") {
          callback(new Error("Request error"));
        }
      }),
    };

    http.request.mockReturnValue(mockReq);

    mockReq.on.mockImplementation((event, callback) => {
      if (event === "data") {
        callback(
          JSON.stringify({ signature: "mock-signature", proof: "mock-proof" })
        );
      }
      if (event === "end") {
        callback();
      }
    });

    tls.connect.mockImplementation(() => ({
      write: jest.fn(),
      on: jest.fn((event, callback) => {
        if (event === "data") {
          callback("mock-tls-data");
        }
        if (event === "end") {
          callback();
        }
      }),
    }));

    await zapProxy.connect();
    const proofResponse = await zapProxy.prove("mock-request");

    expect(proofResponse.signature).toBe("mock-signature");
    expect(proofResponse.proof).toBe("mock-proof");
  });

  test("should handle proof sending errors", async () => {
    tls.connect.mockImplementation(() => ({
      write: jest.fn(),
      on: jest.fn((event, callback) => {
        if (event === "end") {
          callback();
        }
      }),
    }));

    const mockReq = {
      write: jest.fn(),
      end: jest.fn(),
      on: jest.fn(),
    };

    http.request.mockReturnValue(mockReq);

    mockReq.on.mockImplementation((event, callback) => {
      if (event === "end") {
        callback();
      }
    });

    await zapProxy.connect();

    mockReq.on.mockImplementationOnce((event, callback) => {
      if (event === "error") {
        callback(new Error("Request error"));
      }
    });

    await expect(zapProxy.prove("mock-request")).rejects.toThrow(
      "Request error"
    );
  });
});
