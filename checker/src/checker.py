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
    flag_variants = 1
    noise_variants = 1
    havoc_variants = 2
    service_name = "n0t3b00k"
    port = 2323  # The port will automatically be picked up as default by self.connect and self.http.
    ##### END CHECKER PARAMETERS


    def register_user(self, conn, username, password):
        conn.write(f"reg {username} {password}\n")
        self.debug(f"Sent command to register user: {username} with password: {password}")
        is_ok = conn.read_until('>')
        if not 'User successfully registered'.encode() in is_ok:
            raise BrokenServiceException("Failed to register user")

    def login_user(self, conn, username, password):
        conn.write(f"log {username} {password}\n")
        self.debug(f"Sent command to login.")
        is_ok = conn.read_until('>')
        if not 'Successfully logged in!'.encode() in is_ok:
            raise BrokenServiceException("Failed to login")

    def putflag(self):  # type: () -> None
        """
            This method stores a flag in the service.
            In case multiple flags are provided, self.variant_id gives the appropriate index.
            The flag itself can be retrieved from self.flag.
            On error, raise an Eno Exception.
            :raises EnoException on error
            :return this function can return a result if it wants
                    if nothing is returned, the service status is considered okay.
                    the preferred way to report errors in the service is by raising an appropriate enoexception
        """
        try:
            if self.variant_id == 0:
                # Create a TCP connection to the service.
                conn = self.connect()
                welcome = conn.read_until(">")

                # First we need to register a user. So let's create some random strings. (Your real checker should use some funny usernames or so)
                username = ''.join(random.choices(string.ascii_uppercase + string.digits, k=12))
                password = ''.join(random.choices(string.ascii_uppercase + string.digits, k=12))

                self.register_user(conn, username, password)

                # Now we need to login
                self.login_user(conn, username, password)

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


                self.chain_db = {
                    "username": username,
                    "password": password,
                    "noteId": noteId
                }

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
            if self.variant_id == 0:
                # First we check if the previous putflag succeeded!
                try:
                    username = self.chain_db["username"]
                    password = self.chain_db["password"]
                    noteId = self.chain_db["noteId"]
                except IndexError:
                    raise BrokenServiceException("Checked flag was not successfully deployed")

                conn = self.connect()
                welcome = conn.read_until(">")

                # Let's login to the service
                self.login_user(conn, username, password)

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

    def putnoise(self):  # type: () -> None
        """
        This method stores noise in the service. The noise should later be recoverable.
        The difference between noise and flag is, that noise does not have to remain secret for other teams.
        This method can be called many times per round. Check how often using self.variant_id.
        On error, raise an EnoException.
        :raises EnoException on error
        :return this function can return a result if it wants
                if nothing is returned, the service status is considered okay.
                the preferred way to report errors in the service is by raising an appropriate enoexception
        """
        try:
            if self.variant_id == 0:
                conn = self.connect()
                welcome = conn.read_until(">")

                # First we need to register a user. So let's create some random strings. (Your real checker should use some funny usernames or so)
                username = ''.join(random.choices(string.ascii_uppercase + string.digits, k=12))
                password = ''.join(random.choices(string.ascii_uppercase + string.digits, k=12))
                randomNote = ''.join(random.choices(string.ascii_uppercase + string.digits, k=36))

                self.register_user(conn, username, password)

                # Now we need to login
                self.login_user(conn, username, password)

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

                self.chain_db = {
                    "username": username,
                    "password": password,
                    "noteId": noteId,
                    "note": note
                }

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
        This method can be called many times per round. Check how often using variant_id.
        On error, raise an EnoException.
        :raises EnoException on error
        :return this function can return a result if it wants
                if nothing is returned, the service status is considered okay.
                the preferred way to report errors in the service is by raising an appropriate enoexception
        """
        try:
            if self.variant_id == 0:
                try:
                    username = self.chain_db["username"]
                    password = self.chain_db["password"]
                    noteId = self.chain_db["noteId"]
                    randomNote = self.chain_db["note"]
                except IndexError:
                    raise BrokenServiceException("Checked flag was not successfully deployed")

                conn = self.connect()
                welcome = conn.read_until(">")

                # Let's login to the service
                self.login_user(conn, username, password)

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
        try:
            conn = self.connect()
            welcome = conn.read_until(">")

            if self.variant_id == 0:
                # In variant 1, we'll check if the help text is available
                conn.write(f"help\n")
                self.debug(f"Sent help command")
                is_ok = conn.read_until('>')
                for line in ['This is a notebook service. Commands:', 'reg USER PW - Register new account', 'log USER PW - Login to account', 'set TEXT..... - Set a note', 'user  - List all users', 'list - List all notes', 'exit - Exit!', 'dump - Dump the database', 'get ID']:
                    if not line.encode() in is_ok:
                        raise BrokenServiceException("Failed to login")

            elif self.variant_id == 1:
                # In variant 2, we'll check if the `user` command still works.
                username = ''.join(random.choices(string.ascii_uppercase + string.digits, k=12))
                password = ''.join(random.choices(string.ascii_uppercase + string.digits, k=12))

                self.register_user(conn, username, password)
                self.login_user(conn, username, password)

                conn.write(f"user\n")
                self.debug(f"Sent user command")
                is_ok = conn.read_until('>')
                if not 'User 0: '.encode() in is_ok:
                    raise BrokenServiceException("User command does not return any users")

                if username:
                    if not username.encode() in is_ok:
                        raise BrokenServiceException("Flag username not in user output")

            else:
                raise EnoException("Got a unknown variant id")

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