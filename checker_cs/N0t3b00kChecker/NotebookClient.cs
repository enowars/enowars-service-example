using EnoCore.Checker;
using EnoCore.CheckerUtil;
using EnoCore.Models;
using Microsoft.Extensions.Logging;
using N0t3b00kChecker.Db;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net.Sockets;
using System.Text;
using System.Threading;
using System.Threading.Tasks;

namespace N0t3b00kChecker
{
    class NotebookClient : IDisposable
    {
        private static readonly string welcomeMessage = "Welcome to the 1337 n0t3b00k!";
        private readonly ILogger<NotebookClient> logger;
        private NotebookUser? user;
        private EnoCheckerTcpConnection? tcpConnection;

        public NotebookClient(ILogger<NotebookClient> logger)
        {
            this.logger = logger;
        }

        public async Task Connect(CheckerTaskMessage task, CancellationToken token)
        {
            this.logger.LogInformation("Connecting to service");
            this.tcpConnection = await EnoCheckerTcpConnection.Connect(task.Address, Checker.SERVICE_PORT, this.logger, token);

            this.logger.LogDebug("Wait for welcome message");
            var response = await this.tcpConnection.ReceiveUntilAsync(Encoding.ASCII.GetBytes("\n> "), logger, token, errorMessage: "No welcome message received");
            string responseString = Encoding.ASCII.GetString(response);

            if (responseString != welcomeMessage)
            {
                this.logger.LogWarning($"Received no welcome message: {response}");
                throw new MumbleException("No welcome message received");
            }
        }

        public async Task Register(NotebookUser user, CancellationToken token)
        {
            this.logger.LogInformation($"Registering user {user.Username}");
            this.user = user;
            this.logger.LogDebug($"reg {this.user.Username} {this.user.Password}");
            await this.tcpConnection!.SendAsync(Encoding.ASCII.GetBytes($"reg {this.user.Username} {this.user.Password}\n"), logger, token, errorMessage: "Connection error during registration");

            var responseBytes = await this.tcpConnection.ReceiveUntilAsync(Encoding.ASCII.GetBytes("\n> "), logger, token, errorMessage: "Connection error during registration");
            string response = Encoding.ASCII.GetString(responseBytes);

            if (response != "User successfully registered")
            {
                this.logger.LogWarning($"Unexpected response: {response}");
                throw new MumbleException("Registration failed");
            }
        }

        public async Task Login(NotebookUser user, CancellationToken token)
        {
            this.user = user;

            this.logger.LogDebug($"log {this.user.Username} {this.user.Password}");
            await this.tcpConnection!.SendAsync(Encoding.ASCII.GetBytes($"log {this.user.Username} {this.user.Password}\n"), this.logger, token, errorMessage: "Connection error during login");

            var responseBytes = await this.tcpConnection.ReceiveUntilAsync(Encoding.ASCII.GetBytes("\n> "), logger, token, errorMessage: "Connection error during login");
            string response = Encoding.ASCII.GetString(responseBytes);

            if (response != "Successfully logged in!")
            {
                this.logger.LogWarning($"Unexpected response: {response}");
                throw new MumbleException("Login failed");
            }
        }

        public async Task<string> SetNote(string note, CancellationToken token)
        {
            this.logger.LogDebug($"set {note}");
            await this.tcpConnection!.SendAsync(Encoding.ASCII.GetBytes($"set {note}\n"), this.logger, token, errorMessage: "Connection error during set note");

            var responseBytes = await this.tcpConnection.ReceiveUntilAsync(Encoding.ASCII.GetBytes("\n> "), logger, token, errorMessage: "Connection error during set note");
            string response = Encoding.ASCII.GetString(responseBytes);

            try
            {
                var split = response?.Split("Note saved! ID is ");
                return split![1].Substring(0, 32);
            }
            catch (Exception e)
            {
                this.logger.LogWarning($"Unexpected response: {response} {e.Message}\n{e.StackTrace}");
                throw new MumbleException("Set note failed");
            }
        }

        public async Task<string> GetNote(string noteId, CancellationToken token)
        {
            this.logger.LogDebug($"get {noteId}");
            await this.tcpConnection!.SendAsync(Encoding.ASCII.GetBytes($"get {noteId}\n"), this.logger, token, errorMessage: "Connection error during get note");

            var responseBytes = await this.tcpConnection.ReceiveUntilAsync(Encoding.ASCII.GetBytes("\n> "), logger, token, errorMessage: "Connection error during set note");
            string response = Encoding.ASCII.GetString(responseBytes);

            if (response == null)
            {
                throw new MumbleException("Did not receive note");
            }

            return response;
        }

        public async Task<string> GetHelp(CancellationToken token)
        {
            this.logger.LogDebug($"GetHelp");
            await this.tcpConnection!.SendAsync(Encoding.ASCII.GetBytes($"help\n"), this.logger, token, errorMessage: "Connection error during help");

            var responseBytes = await this.tcpConnection.ReceiveUntilAsync(Encoding.ASCII.GetBytes("\n> "), logger, token, errorMessage: "Connection error during set note");
            return Encoding.ASCII.GetString(responseBytes);
        }

        public async Task<List<string>> GetUsers(CancellationToken token)
        {
            this.logger.LogDebug($"GetUsers");
            await this.tcpConnection!.SendAsync(Encoding.ASCII.GetBytes($"user\n"), this.logger, token, errorMessage: "Connection error during help");

            var responseBytes = await this.tcpConnection.ReceiveUntilAsync(Encoding.ASCII.GetBytes("\n> "), logger, token, errorMessage: "Connection error during set note");
            var response = Encoding.ASCII.GetString(responseBytes);
            var users = new List<string>();
            try
            {
                foreach (var userEntry in response.Split('\n'))
                {
                    users.Add(userEntry.Split(": ")[1]);
                }
            }
            catch (Exception e)
            {
                this.logger.LogWarning($"Get note failed: {e.Message}\n{e.StackTrace}");
                throw new MumbleException("Invalid user list");
            }
            return users;
        }

        public async Task<List<string>> GetNotes(CancellationToken token)
        {
            this.logger.LogDebug($"GetList");
            await this.tcpConnection!.SendAsync(Encoding.ASCII.GetBytes($"list\n"), this.logger, token, errorMessage: "Connection error during help");

            var responseBytes = await this.tcpConnection.ReceiveUntilAsync(Encoding.ASCII.GetBytes("\n> "), logger, token, errorMessage: "Connection error during set note");
            var response = Encoding.ASCII.GetString(responseBytes);
            var notes = new List<string>();
            try
            {
                foreach (var noteEntry in response.Split('\n'))
                {
                    notes.Add(noteEntry.Split(": ")[1]);
                }
            }
            catch (Exception e)
            {
                this.logger.LogWarning($"Get note failed: {e.Message}\n{e.StackTrace}");
                throw new OfflineException("Connection error during set note");
            }
            return notes;
        }

        public void Dispose()
        {
            this.tcpConnection?.Dispose();
        }
    }
}
