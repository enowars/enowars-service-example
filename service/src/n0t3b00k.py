import socket
import threading
import hashlib
import os
import json

#### Service tenets
# A service SHOULD NOT be a simple wrapper for a key-value database, and SHOULD expose more complex functionality => This is an example service, so it does not satisfy this tenet.
# Rewriting a service with the same feature set SHOULD NOT be feasible within the timeframe of the contest => Same as above
# A service MAY be written in unexpected languages or using fun frameworks => Same as above
# A service MUST have at least one complex vulnerability => Hm, maybe the path traversal qualifies
# Vulnerabilities SHOULD NOT be easily replayable => This is an example service.
# A service SHOULD NOT have unintended vulnerabilities => Hm, maybe reading flags or files?
####


# This is a quick & dirty class to persist our notes to the filesystem instead of memory.
class FilesystemDict(dict):
    def __init__(self, folder, *args, **kwargs):
        self.folder = folder
        super(FilesystemDict, self).__init__(*args, **kwargs)
        if not os.path.exists(self.folder):
            os.mkdir(self.folder)

    def __setitem__(self, key, item):
        with open(os.path.join(self.folder, key), "w") as f:
            f.write(json.dumps(item))

    def __getitem__(self, key):
        with open(os.path.join(self.folder, key), "r") as f:
            return json.loads(f.read())

    def __repr__(self):
        return f"FilesystemDict @ {self.folder}"

    def __len__(self):
        _, _, files = next(os.walk(self.folder))
        return len(files) - 1

    def __delitem__(self, key):
        if os.path.exists(os.path.join(self.folder, key)):
            os.unlink(os.path.join(self.folder, key))

    def clear(self):
        raise Exception("Not supported")

    def copy(self):
        raise Exception("Not supported")

    def has_key(self, key):
        return os.path.exists(os.path.join(self.folder, key))

    def update(self, *args, **kwargs):
        raise Exception("Not supported")

    def keys(self):
        _, _, files = next(os.walk(self.folder))
        return files

    def values(self):
        values = []
        for key in self.keys():
            values.append(self.__getitem__(key))

    def items(self):
        return self.__dict__.items()

    def pop(self, *args):
        raise Exception("Not supported")

    def __cmp__(self, dict_):
        raise Exception("Not supported")

    def __contains__(self, item):
        raise Exception("Not supported")

    def __iter__(self):
        raise Exception("Not supported")

    def __unicode__(self):
        raise Exception("Not supported")


class ThreadedServer(object):
    def __init__(self, host, port):
        self.host = host
        self.port = port
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self.sock.bind((self.host, self.port))
        self.acceptingThread = None


        self.users = FilesystemDict(folder="/data/users/")
        self.notes = FilesystemDict(folder="/data/notes/")
        self.userNotes = FilesystemDict(folder="/data/userNotes/")
        self.debug = True

    def listen(self):
        self.acceptingThread = threading.Thread(target = self.accept)
        self.acceptingThread.start()
        return self.acceptingThread

    def accept(self):
        self.sock.listen(10)
        while True:
            client, address = self.sock.accept()
            client.settimeout(60)
            threading.Thread(target = self.listenToClient,args = (client,address)).start()

    def listenToClient(self, client, address):
        try:
            ch = client.makefile(mode="r")
            client.send("Welcome to the 1337 n0t3b00k!\n".encode())
            line = ""
            currUser = None
            while line != "exit":
                client.send("> ".encode())
                line = ch.readline()
                line = line.strip()
                if line in ["?", "help"] or len(line) < 4:
                    client.send("""
This is a notebook service. Commands:
reg USER PW - Register new account
log USER PW - Login to account
set TEXT..... - Set a note
user  - List all users
list - List all notes
exit - Exit!
dump - Dump the database
get ID\n""".encode())
                elif line[:4] == "user":
                    for i, u in enumerate(self.users.keys()):
                        client.send("User {}: {}\n".format(i, u).encode())
                    continue
                elif line[:3] == "reg":
                    split = line[4:].split(" ")
                    reg_user = split[0]
                    reg_pw = split[1]
                    self.users[reg_user] = reg_pw
                    if not self.userNotes.has_key(reg_user):
                        self.userNotes[reg_user] = []
                    client.send("User successfully registered\n".encode())
                    continue
                elif line[:3] == "log":
                    split = line[4:].split(" ")
                    log_usr = split[0]
                    log_pw = split[1]
                    if not log_usr in self.users.keys():
                        client.send("User not found!\n".encode())
                        continue
                    if self.users[log_usr] != log_pw:
                        client.send("Wrong password!\n".encode())
                        continue
                    currUser = log_usr
                    client.send("Successfully logged in!\n".encode())
                    continue
                elif line[:3] == "set":
                    if currUser == None:
                        client.send("Not logged in!\n".encode())
                        continue
                    data = line[4:]
                    h = hashlib.md5(data.encode()).hexdigest()
                    self.notes[h] = data
                    userNotes = self.userNotes[currUser]
                    userNotes.append(h)
                    self.userNotes[currUser] = userNotes
                    client.send("Note saved! ID is ".encode() + h.encode() +  "!\n".encode())
                    continue
                elif line[:3] == "get":
                    h = line[4:]
                    if not h in self.notes.keys():
                        client.send("This note does not exist!\n".encode())
                        continue
                    data = self.notes[h]
                    client.send(data.encode() + "\n".encode())
                    continue 
                elif line[:4] == "list":
                    if currUser == None:
                        client.send("Not logged in!\n".encode())
                        continue
                    for i, d in enumerate(self.userNotes[currUser]):
                        client.send("Note {}: {}\n".format(i, d).encode())
                    continue
                elif line[:4] == "dump":
                    if not self.debug:
                        continue
                    client.send("Users:\n".encode())
                    for user in self.users.keys():
                        client.send("{}:{}\n".format(user, self.users[user]).encode())
                        for i, d in enumerate(self.userNotes[user]):
                            client.send("\t Note {}:{}:{}\n".format(i, d, self.notes[d]).encode())
                    continue

        except Exception as e:
            print(e)
        client.close()

if __name__ == "__main__":
    print("n0t3b00k starting!")
    service = ThreadedServer('',8000).listen()
    print("ThreadedServer listening...")
    service.join()
    print("n0t3b00k exiting.")
