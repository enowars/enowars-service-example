#!/usr/bin/env python3
from enochecker import BaseChecker, BrokenServiceException, EnoException, run
from enochecker.utils import SimpleSocket, assert_equals, assert_in
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

    def register_user(self, conn: SimpleSocket, username: str, password: str):
        conn.write(f"reg {username} {password}\n")
        self.debug(
            f"Sent command to register user: {username} with password: {password}"
        )
        conn.readline_expect(
            b"User successfully registered",
            read_until=b">",
            exception_message="Failed to register user",
        )

    def login_user(self, conn: SimpleSocket, username: str, password: str):
        conn.write(f"log {username} {password}\n")
        self.debug(f"Sent command to login.")
        conn.readline_expect(
            b"Successfully logged in!",
            read_until=b">",
            exception_message="Failed to log in",
        )

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
        if self.variant_id == 0:
            # Create a TCP connection to the service.
            conn = self.connect()
            welcome = conn.read_until(">")

            # First we need to register a user. So let's create some random strings. (Your real checker should use some funny usernames or so)
            username: str = "".join(
                random.choices(string.ascii_uppercase + string.digits, k=12)
            )
            password: str = "".join(
                random.choices(string.ascii_uppercase + string.digits, k=12)
            )

            self.register_user(conn, username, password)

            # Now we need to login
            self.login_user(conn, username, password)

            # Finally, we can post our note!
            conn.write(f"set {self.flag}\n")
            self.debug(f"Sent command to set the flag")
            conn.read_until(b"Note saved! ID is ")
            try:
                noteId = conn.read_until(b"!\n>").rstrip(b"!\n>").decode()
            except Exception as ex:
                self.debug(f"Failed to retrieve note: {ex}")
                raise BrokenServiceException("Could not retrieve NoteId")

            assert_equals(len(noteId) > 0, True, message="Empty noteId received")

            self.debug(f"Got noteId {noteId}")

            # Exit!
            conn.write(f"exit\n")
            conn.close()

            self.chain_db = {
                "username": username,
                "password": password,
                "noteId": noteId,
            }
        else:
            raise EnoException("Wrong variant_id provided")

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
        if self.variant_id == 0:
            # First we check if the previous putflag succeeded!
            try:
                username: str = self.chain_db["username"]
                password: str = self.chain_db["password"]
                noteId: bytes = self.chain_db["noteId"]
            except IndexError as ex:
                self.debug(f"error getting notes from db: {ex}")
                raise EnoException("Failed to read DB")

            conn = self.connect()
            welcome = conn.read_until(">")

            # Let's login to the service
            self.login_user(conn, username, password)

            # Let´s obtain our note.
            conn.write(f"get {noteId}\n")
            self.debug(f"Sent command to retrieve note: {noteId}")
            note = conn.read_until(">")
            assert_in(
                self.flag.encode(), note, "Resulting flag was found to be incorrect"
            )

            # Exit!
            conn.write(f"exit\n")
            conn.close()
        else:
            raise EnoException("Wrong variant_id provided")


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
        if self.variant_id == 0:
            conn = self.connect()
            welcome = conn.read_until(">")

            # First we need to register a user. So let's create some random strings. (Your real checker should use some funny usernames or so)
            username = "".join(
                random.choices(string.ascii_uppercase + string.digits, k=12)
            )
            password = "".join(
                random.choices(string.ascii_uppercase + string.digits, k=12)
            )
            randomNote = "".join(
                random.choices(string.ascii_uppercase + string.digits, k=36)
            )

            self.register_user(conn, username, password)

            # Now we need to login
            self.login_user(conn, username, password)

            # Finally, we can post our note!
            conn.write(f"set {randomNote}\n")
            self.debug(f"Sent command to set the flag")
            self.read_until(b"Note saved! ID is ")
            try:
                noteId = conn.read_until(b"!\n>").rstrip(b"!\n>").decode()
            except Exception as ex:
                self.debug(f"Failed to retrieve note: {ex}")
                raise BrokenServiceException("Could not retrieve NoteId")

            assert_equals(len(noteId) > 0, True, message="Empty noteId received")

            self.debug(f"{noteId}")

            # Exit!
            conn.write(f"exit\n")
            conn.close()

            self.chain_db = {
                "username": username,
                "password": password,
                "noteId": noteId,
                "note": randomNote,
            }
        else:
            raise EnoException("Wrong variant_id provided")

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
        if self.variant_id == 0:
            try:
                username: str = self.chain_db["username"]
                password: str = self.chain_db["password"]
                noteId: str = self.chain_db["noteId"]
                randomNote: str = self.chain_db["note"]
            except Exception as ex:
                self.debug("Failed to read db {ex}")
                raise EnoException("Failed to read DB")

            conn = self.connect()
            welcome = conn.read_until(">")

            # Let's login to the service
            self.login_user(conn, username, password)

            # Let´s obtain our note.
            conn.write(f"get {noteId}\n")
            self.debug(f"Sent command to retrieve note: {noteId}")
            note = conn.read_until(">")
            conn.readline_expect(
                randomNote.encode(), ">", "Resulting flag was found to be incorrect"
            )

            # Exit!
            conn.write(f"exit\n")
            conn.close()
        else:
            raise EnoException("Wrong variant_id provided")

    def havoc(self):  # type: () -> None
        """
        This method unleashes havoc on the app -> Do whatever you must to prove the service still works. Or not.
        On error, raise an EnoException.
        :raises EnoException on Error
        :return This function can return a result if it wants
                If nothing is returned, the service status is considered okay.
                The preferred way to report Errors in the service is by raising an appropriate EnoException
        """
        conn = self.connect()
        welcome = conn.read_until(">")

        if self.variant_id == 0:
            # In variant 1, we'll check if the help text is available
            conn.write(f"help\n")
            self.debug(f"Sent help command")
            is_ok = conn.read_until(">")
            # TODO: probably should check if this is in the correct order
            for line in [
                "This is a notebook service. Commands:",
                "reg USER PW - Register new account",
                "log USER PW - Login to account",
                "set TEXT..... - Set a note",
                "user  - List all users",
                "list - List all notes",
                "exit - Exit!",
                "dump - Dump the database",
                "get ID",
            ]:
                assert_in(line.encode(), is_ok, "Received incomplete response.")

        elif self.variant_id == 1:
            # In variant 2, we'll check if the `user` command still works.
            username = "".join(
                random.choices(string.ascii_uppercase + string.digits, k=12)
            )
            password = "".join(
                random.choices(string.ascii_uppercase + string.digits, k=12)
            )

            self.register_user(conn, username, password)
            self.login_user(conn, username, password)

            conn.write(f"user\n")
            self.debug(f"Sent user command")
            ret = conn.readline_expect(
                "User 0: ",
                read_until=b">",
                exception_message="User command does not return any users",
            )

            if username:
                assert_in(username.encode(), ret, "Flag username not in user output")

        else:
            raise EnoException("Wrong variant_id provided")

        # Exit!
        conn.write(f"exit\n")
        conn.close()

    def exploit(self):
        """
        This method was added for CI purposes for exploits to be tested.
        Will (hopefully) not be called during actual CTF.
        :raises EnoException on Error
        :return This function can return a result if it wants
                If nothing is returned, the service status is considered okay.
                The preferred way to report Errors in the service is by raising an appropriate EnoException
        """
        # TODO: Add your exploit here.
        pass


app = N0t3b00kChecker.service  # This can be used for uswgi.
if __name__ == "__main__":
    run(N0t3b00kChecker)
