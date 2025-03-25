from http.server import ThreadingHTTPServer
from Server import Server

hostName = "localhost"
serverPort = 8080

if __name__ == "__main__":        
    webServer = ThreadingHTTPServer((hostName, serverPort), Server)
    print("Server started http://%s:%s" % (hostName, serverPort))

    try:
        webServer.serve_forever()
    except KeyboardInterrupt:
        pass

    webServer.server_close()
    print("Server stopped.")