from http.server import BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs
import logging
import json
import time
from Diagrams import Diagrams

diagrams = Diagrams("data/diagrams.json")
class Server(BaseHTTPRequestHandler):
    def do_GET(self): 
        (apiPath, query) = self.isAPI()
        if apiPath:
            if apiPath == ["diagrams"]:
                prevTime = float(query["time"][0])
                nowTime = time.time()
                allDiagrams = diagrams.getDiagrams()
                self.sendJSON({
                    "diagrams": [
                        {
                            "name": diagram["name"],
                            "type": diagram["type"],
                            "diagram": diagram["diagram"] if diagram["date"] > prevTime else False,
                            "state": diagram["state"] if diagram["date"] > prevTime else False,
                        } for diagram in allDiagrams
                    ], 
                    "time": nowTime
                })
            else: 
                self.send_response(200)
                self.send_header("Content-type", "text/html")
                self.send_header( "Access-Control-Allow-Origin", "*")
                self.end_headers()
                self.wfile.write(bytes("<body>Good, this is the API</body>", "utf-8"))
        else:
            self.send_response(200)
            self.send_header("Content-type", "text/html")
            self.send_header( "Access-Control-Allow-Origin", "*")
            self.end_headers()
            self.wfile.write(bytes("<html><head><title>https://pythonbasics.org</title></head>", "utf-8"))
            self.wfile.write(bytes("<p>Request: %s</p>" % self.path, "utf-8"))
            self.wfile.write(bytes("<body>", "utf-8"))
            self.wfile.write(bytes("<p>This is an example web servers.</p>", "utf-8"))
            self.wfile.write(bytes("</body></html>", "utf-8"))

    def do_POST(self): 
        (apiPath, query) = self.isAPI()
        try: 
            self.send_response(200)
            if apiPath:
                contentLen = int(self.headers.get('Content-Length'))
                postBody = self.rfile.read(contentLen)
                bodyText = postBody.decode("utf-8")
                if apiPath == ["diagram"]:
                    name = query["name"][0]
                    type = query["type"][0]
                    diagrams.removeDiagram(name)
                    diagrams.addDiagram(name, type, bodyText)
                    diagrams.saveToFile()
                if apiPath == ["diagramState"]:
                    name = query["name"][0]
                    diagrams.setDiagramState(name, bodyText)
                    diagrams.saveToFile()
            else: 
                self.send_header("Content-type", "text/html")
        except Exception as error: 
            logging.exception(error)
            self.send_response(500)
        finally:
            self.send_header( "Access-Control-Allow-Origin", "*")
            self.end_headers()

    def do_DELETE(self):
        (apiPath, query) = self.isAPI()
        try: 
            self.send_response(200)
            if apiPath:
                if apiPath == ["diagram"]:
                    name = query["name"][0]
                    diagrams.removeDiagram(name)
                    diagrams.saveToFile()
            else: 
                self.send_header("Content-type", "text/html")
        except Exception as error: 
            logging.exception(error)
            self.send_response(500)
        finally:
            self.send_header( "Access-Control-Allow-Origin", "*")
            self.end_headers()

    def sendJSON(self, object):
        self.send_response(200)
        self.send_header("Content-type", "application/json")
        self.send_header( "Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(bytes(json.dumps(object), "utf-8"))


    def isAPI(self):
        parsed = urlparse(self.path)
        pathParts = parsed.path.split("/")[1::]
        if pathParts[0] == "api":
            return (pathParts[1::], parse_qs(parsed.query))
        return (False, parse_qs(parsed.query))
    
    # Needed for cors on chrome:
    def do_OPTIONS(self):
        self.send_response(200)
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, OPTIONS, DELETE, POST')
        self.send_header("Access-Control-Allow-Headers", "X-Requested-With")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.end_headers()
