import json
import os
import time
import logging

class Diagrams:
    def __init__(self, path):
        self.filePath = os.path.join(os.getcwd(), path)
        self.dir = os.path.split(self.filePath)[0]
        os.makedirs(self.dir, exist_ok=True)
        self.loadFromFile()

    def loadFromFile(self):
        self.diagrams = self.readFromFile()

    def readFromFile(self):
        outDiagrams = []
        try:
            diagrams = json.load(open(self.filePath, "r"))
            for diagramData in diagrams:
                try:
                    diagramPath = os.path.join(self.dir, diagramData["diagram"])
                    diagramStatePath = os.path.join(self.dir, diagramData["state"])
                    diagram = open(diagramPath, "r", encoding='utf-8').read()
                    state = open(diagramStatePath, "r", encoding='utf-8').read()
                    outDiagrams.append({
                        "name": diagramData["name"],
                        "type": diagramData["type"],
                        "date": diagramData["date"],
                        "diagram": diagram,
                        "state": state
                    })
                except Exception as error: logging.exception(error)
        except Exception as error: logging.exception(error)
        return outDiagrams
    
    def getDiagrams(self):
        return self.diagrams

    def addDiagram(self, name, type, diagram):
        self.diagrams.append({
            "name": name,
            "type": type,
            "date": time.time(),
            "diagram": diagram,
            "state": "",
        })

    def setDiagramState(self, name, state):
        for diagramData in self.diagrams:
            if diagramData["name"] == name:
                diagramData["state"] = state

    def removeDiagram(self, name):
        self.diagrams = [d for d in self.diagrams if d["name"]!=name]

    def saveToFile(self):
        try: 
            for diagramData in json.load(open(self.filePath, "r")):
                try:
                    os.remove(os.path.join(self.dir, diagramData["diagram"]))
                    os.remove(os.path.join(self.dir, diagramData["state"]))
                except Exception as error: logging.exception(error)
        except Exception as error: logging.exception(error)

        outDiagrams = []
        for diagramData in self.diagrams:
            diagramFilePath = diagramData["name"]+".txt"
            with open(os.path.join(self.dir, diagramFilePath), "w+", encoding='utf-8') as diagramFile:
                diagramFile.write(diagramData["diagram"])

            stateFilePath = diagramData["name"]+"_state.json"
            with open(os.path.join(self.dir, stateFilePath), "w+", encoding='utf-8') as stateFile:
                stateFile.write(diagramData["state"])

            outDiagrams.append({
                "name": diagramData["name"], 
                "type": diagramData["type"],
                "date": diagramData["date"],
                "diagram": diagramFilePath,
                "state": stateFilePath
            })

        with open(self.filePath, "w+") as diagramsFile:
            diagramsFile.write(json.dumps(outDiagrams))

        