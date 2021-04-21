#!/usr/bin/env python3
from enochecker import *
import random
import string

class N0t3b00kChecker(BaseChecker):
    """
    Change the methods given here, then simply create the class and .run() it.
    Magic.
    A few convenient methods and helpers are provided in the BaseChecker.
    ensure_bytes ans ensure_unicode to make sure strings are always equal.
    As well as methods:
    self.connect() connects to the remote server.
    self.get and self.post request from http.
    self.team_db is a dict that stores its contents to filesystem. (call .persist() to make sure)
    self.readline_expect(): fails if it's not read correctly
    To read the whole docu and find more goodies, run python -m pydoc enochecker
    (Or read the source, Luke)
    """

    ##### EDIT YOUR CHECKER PARAMETERS
    flag_count = 1
    noise_count = 1
    havoc_count = 0
    service_name = "n0t3b00k"
    port = 2323  # The port will automatically be picked up as default by self.connect and self.http.
    ##### END CHECKER PARAMETERS


    def flag_key(self):
        """
            This function creates a unique key for our mongo db based on the flag.
        """
        return f"flag_{self.flag_round}:{self.flag_idx}"

    def putflag(self):  # type: () -> None
        """
            This method stores a flag in the service.
            In case multiple flags are provided, self.flag_idx gives the appropriate index.
            The flag itself can be retrieved from self.flag.
            On error, raise an Eno Exception.
            :raises EnoException on error
            :return this function can return a result if it wants
                    if nothing is returned, the service status is considered okay.
                    the preferred way to report errors in the service is by raising an appropriate enoexception
        """
        try:
            if self.flag_idx == 0:
                # Create a TCP connection to the service.
                conn = self.connect()

                # First we need to register a user. So let's create some random strings. (Your real checker should use some funny usernames or so)
                username = ''.join(random.choices(string.ascii_uppercase + string.digits, k=12))
                password = ''.join(random.choices(string.ascii_uppercase + string.digits, k=12))

                welcome = conn.read_until(">")
                conn.write(f"reg {username} {password}\n")
                self.debug(f"Sent command to register user: {username} with password: {password}")
                is_ok = conn.read_until('>')
                if not 'User successfully registered'.encode() in is_ok:
                    raise BrokenServiceException("Failed to register user")

                # Now we need to login
                conn.write(f"log {username} {password}\n")
                self.debug(f"Sent command to login.")
                is_ok = conn.read_until('>')
                if not 'Successfully logged in!'.encode() in is_ok:
                    raise BrokenServiceException("Failed to login")

                # Finally, we can post our note!
                conn.write(f"set {self.flag}\n")
                self.debug(f"Sent command to set the flag")
                is_ok = conn.read_until('>')
                if not 'Note saved! ID is '.encode() in is_ok:
                    raise BrokenServiceException("Failed to save note!")

                noteId = is_ok.decode().strip().lstrip('Note saved! ID is ').rstrip("!\n>") # This is hacky and should not be done in production-grade checkers.
                self.debug(f"{noteId}")

                # Exit!
                conn.write(f"exit\n")
                conn.close()

                self.team_db[self.flag_key() + "username"] = username
                self.team_db[self.flag_key() + "password"] = password
                self.team_db[self.flag_key() + "noteId"] = noteId

        except EOFError:
            raise OfflineException("Encountered unexpected EOF")
        except UnicodeError:
            self.debug("UTF8 Decoding-Error")
            raise BrokenServiceException("Fucked UTF8")


    def getflag(self):  # type: () -> None
        """
        This method retrieves a flag from the service.
        Use self.flag to get the flag that needs to be recovered and self.round to get the round the flag was placed in.
        On error, raise an EnoException.
        :raises EnoException on error
        :return this function can return a result if it wants
                if nothing is returned, the service status is considered okay.
                the preferred way to report errors in the service is by raising an appropriate enoexception
        """
        try:
            if self.flag_idx == 0:
                # First we check if the previous putflag succeeded!
                try:
                    username = self.team_db[self.flag_key() + "username"]
                    password = self.team_db[self.flag_key() + "password"]
                    noteId = self.team_db[self.flag_key() + "noteId"]
                except IndexError:
                    raise BrokenServiceException("Checked flag was not successfully deployed")

                conn = self.connect()

                # Let's login to the service
                welcome = conn.read_until(">")
                conn.write(f"log {username} {password}\n")
                self.debug(f"Sent command to login user: {username} with password: {password}")
                is_ok = conn.read_until('>')
                if not 'Successfully logged in!'.encode() in is_ok:
                    raise BrokenServiceException("Failed to login")

                # Let´s obtain our note.
                conn.write(f"get {noteId}\n")
                self.debug(f"Sent command to retrieve note: {noteId}")
                note = conn.read_until(">")
                if not self.flag.encode() in note:
                    self.debug(f"Flags do not match: {self.flag.encode()}  vs. {note}")
                    raise BrokenServiceException("Resulting flag was found to be incorrect")

                # Exit!
                conn.write(f"exit\n")
                conn.close()

        except EOFError:
            raise OfflineException("Encountered unexpected EOF")
        except UnicodeError:
            self.debug("UTF8 Decoding-Error")
            raise BrokenServiceException("Fucked UTF8")

    def noise_key(self):
        """
            This function creates a unique key for our mongo db based on the flag.
        """
        return f"noise_{self.flag_round}:{self.flag_idx}"

    def putnoise(self):  # type: () -> None
        """
        This method stores noise in the service. The noise should later be recoverable.
        The difference between noise and flag is, that noise does not have to remain secret for other teams.
        This method can be called many times per round. Check how often using self.flag_idx.
        On error, raise an EnoException.
        :raises EnoException on error
        :return this function can return a result if it wants
                if nothing is returned, the service status is considered okay.
                the preferred way to report errors in the service is by raising an appropriate enoexception
        """
        try:
            if self.flag_idx == 0:
                conn = self.connect()

                # First we need to register a user. So let's create some random strings. (Your real checker should use some funny usernames or so)
                username = ''.join(random.choices(string.ascii_uppercase + string.digits, k=12))
                password = ''.join(random.choices(string.ascii_uppercase + string.digits, k=12))
                randomNote = ''.join(random.choices(string.ascii_uppercase + string.digits, k=36))

                welcome = conn.read_until(">")
                conn.write(f"reg {username} {password}\n")
                self.debug(f"Sent command to register user: {username} with password: {password}")
                is_ok = conn.read_until('>')
                if not 'User successfully registered'.encode() in is_ok:
                    raise BrokenServiceException("Failed to register user")

                # Now we need to login
                conn.write(f"log {username} {password}\n")
                self.debug(f"Sent command to login.")
                is_ok = conn.read_until('>')
                if not 'Successfully logged in!'.encode() in is_ok:
                    raise BrokenServiceException("Failed to login")

                # Finally, we can post our note!
                conn.write(f"set {randomNote}\n")
                self.debug(f"Sent command to set the flag")
                is_ok = conn.read_until('>')
                if not 'Note saved! ID is '.encode() in is_ok:
                    raise BrokenServiceException("Failed to save note!")

                noteId = is_ok.decode().strip().lstrip('Note saved! ID is ').rstrip("!\n>") # This is hacky and should not be done in production-grade checkers.
                self.debug(f"{noteId}")

                # Exit!
                conn.write(f"exit\n")
                conn.close()

                self.team_db[self.noise_key() + "username"] = username
                self.team_db[self.noise_key() + "password"] = password
                self.team_db[self.noise_key() + "noteId"] = noteId
                self.team_db[self.noise_key() + "note"] = randomNote

        except EOFError:
            raise OfflineException("Encountered unexpected EOF")
        except UnicodeError:
            self.debug("UTF8 Decoding-Error")
            raise BrokenServiceException("Fucked UTF8")

    def getnoise(self):  # type: () -> None
        """
        This method retrieves noise in the service.
        The noise to be retrieved is inside self.flag
        The difference between noise and flag is, that noise does not have to remain secret for other teams.
        This method can be called many times per round. Check how often using flag_idx.
        On error, raise an EnoException.
        :raises EnoException on error
        :return this function can return a result if it wants
                if nothing is returned, the service status is considered okay.
                the preferred way to report errors in the service is by raising an appropriate enoexception
        """
        try:
            if self.flag_idx == 0:
                try:
                    username = self.team_db[self.noise_key() + "username"]
                    password = self.team_db[self.noise_key() + "password"]
                    noteId = self.team_db[self.noise_key() + "noteId"]
                    randomNote = self.team_db[self.noise_key() + "note"]
                except IndexError:
                    raise BrokenServiceException("Checked flag was not successfully deployed")

                conn = self.connect()

                # Let's login to the service
                welcome = conn.read_until(">")
                conn.write(f"log {username} {password}\n")
                self.debug(f"Sent command to login user: {username} with password: {password}")
                is_ok = conn.read_until('>')
                if not 'Successfully logged in!'.encode() in is_ok:
                    raise BrokenServiceException("Failed to login")

                # Let´s obtain our note.
                conn.write(f"get {noteId}\n")
                self.debug(f"Sent command to retrieve note: {noteId}")
                note = conn.read_until(">")
                if not randomNote.encode() in note:
                    self.debug(f"Flags do not match: {randomNote.encode()}  vs. {note}")
                    raise BrokenServiceException("Resulting flag was found to be incorrect")

                # Exit!
                conn.write(f"exit\n")
                conn.close()

        except EOFError:
            raise OfflineException("Encountered unexpected EOF")
        except UnicodeError:
            self.debug("UTF8 Decoding-Error")
            raise BrokenServiceException("Fucked UTF8")
        except KeyError:
            raise BrokenServiceException("Noise not found!")

    def havoc(self):  # type: () -> None
        """
        This method unleashes havoc on the app -> Do whatever you must to prove the service still works. Or not.
        On error, raise an EnoException.
        :raises EnoException on Error
        :return This function can return a result if it wants
                If nothing is returned, the service status is considered okay.
                The preferred way to report Errors in the service is by raising an appropriate EnoException
        """
        self.info("I wanted to inform you: I'm  running <3")

    def exploit(self):
        """
        This method was added for CI purposes for exploits to be tested.
        Will (hopefully) not be called during actual CTF.
        :raises EnoException on Error
        :return This function can return a result if it wants
                If nothing is returned, the service status is considered okay.
                The preferred way to report Errors in the service is by raising an appropriate EnoException
        """
        pass

app = N0t3b00kChecker.service  # This can be used for uswgi.
if __name__ == "__main__":
    run(N0t3b00kChecker)