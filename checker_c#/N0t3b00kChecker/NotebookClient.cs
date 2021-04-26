using EnoCore.Checker;
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
        private readonly ILogger<NotebookClient> logger;
        private NotebookUser? user;
        private TcpClient? tcpClient;
        private StreamReader? reader;
        private StreamWriter? writer;

        public NotebookClient(ILogger<NotebookClient> logger)
        {
            this.logger = logger;
        }

        public async Task Connect(CheckerTaskMessage task, CancellationToken token)
        {
            this.logger.LogInformation("Connecting to service");
            this.tcpClient = new();
            try
            {
                await this.tcpClient.ConnectAsync(task.Address, Checker.SERVICE_PORT, token);
                this.reader = new StreamReader(this.tcpClient.GetStream());
                this.writer = new StreamWriter(this.tcpClient.GetStream());
                token.Register(() =>
                {
                    tcpClient.Close();
                    reader.Close();
                    writer.Close();
                });
            }
            catch (Exception e)
            {
                this.logger.LogWarning($"Connect failed: {e.Message}\n{e.StackTrace}");
                throw new OfflineException("Failed to establish TCP connection");
            }

            string? response;
            try
            {
                this.logger.LogDebug("Wait for welcome message");
                response = await this.reader!.ReadLineAsync();
                await reader!.ReadAsync(new char[2].AsMemory(), token);
            }
            catch (Exception e)
            {
                this.logger.LogWarning($"Received no welcome message: {e.Message}\n{e.StackTrace}");
                throw new OfflineException("No welcome message received");
            }

            if (response != "Welcome to the 1337 n0t3b00k!")
            {
                this.logger.LogWarning($"Received no welcome message: {response}");
                throw new MumbleException("No welcome message received");
            }
        }

        public async Task Register(NotebookUser user, CancellationToken token)
        {
            this.logger.LogInformation($"Registering user {user.Username}");
            this.user = user;
            try
            {
                this.logger.LogDebug($"reg {this.user.Username} {this.user.Password}");
                await this.writer!.WriteAsync($"reg {this.user.Username} {this.user.Password}\n".AsMemory(), token);
                await this.writer.FlushAsync();
            }
            catch (Exception e)
            {
                this.logger.LogWarning($"Register failed: {e.Message}\n{e.StackTrace}");
                throw new OfflineException("Connection error during registration");
            }

            string? response;
            try
            {
                this.logger.LogDebug("Wait for register success message");
                response = await this.reader!.ReadLineAsync();
                await reader!.ReadAsync(new char[2].AsMemory(), token);
            }
            catch (Exception e)
            {
                this.logger.LogWarning($"Registration failed: {e.Message}\n{e.StackTrace}");
                throw new OfflineException("Connection error during registration");
            }

            if (response != "User successfully registered")
            {
                this.logger.LogWarning($"Unexpected response: {response}");
                throw new MumbleException("Registration failed");
            }
        }

        public async Task Login(NotebookUser user, CancellationToken token)
        {
            this.user = user;
            try
            {
                this.logger.LogDebug($"log {this.user.Username} {this.user.Password}");
                await this.writer!.WriteAsync($"log {this.user.Username} {this.user.Password}\n".AsMemory(), token);
                await this.writer.FlushAsync();
            }
            catch (Exception e)
            {
                this.logger.LogWarning($"Login failed: {e.Message}\n{e.StackTrace}");
                throw new OfflineException("Connection error during login");
            }

            string? response;
            try
            {
                response = await this.reader!.ReadLineAsync();
                await reader!.ReadAsync(new char[2].AsMemory(), token);
            }
            catch (Exception e)
            {
                this.logger.LogWarning($"Login failed: {e.Message}\n{e.StackTrace}");
                throw new OfflineException("Connection error during login");
            }

            if (response != "Successfully logged in!")
            {
                this.logger.LogWarning($"Unexpected response: {response}");
                throw new MumbleException("Login failed");
            }
        }

        public async Task<string> SetNote(string note, CancellationToken token)
        {
            try
            {
                this.logger.LogDebug($"set {note}");
                await this.writer!.WriteAsync($"set {note}\n".AsMemory(), token);
                await this.writer.FlushAsync();
            }
            catch (Exception e)
            {
                this.logger.LogWarning($"Set note failed: {e.Message}\n{e.StackTrace}");
                throw new OfflineException("Connection error during set note");
            }

            string? response;
            try
            {
                response = await this.reader!.ReadLineAsync();
            }
            catch (Exception e)
            {
                this.logger.LogWarning($"Set note failed: {e.Message}\n{e.StackTrace}");
                throw new OfflineException("Connection error during set note");
            }

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
            try
            {
                this.logger.LogDebug($"get {noteId}");
                await this.writer!.WriteAsync($"get {noteId}\n".AsMemory(), token);
                await this.writer.FlushAsync();
            }
            catch (Exception e)
            {
                this.logger.LogWarning($"Get note failed: {e.Message}\n{e.StackTrace}");
                throw new OfflineException("Connection error during get note");
            }

            string? response;
            try
            {
                response = await this.reader!.ReadLineAsync();
                await reader!.ReadAsync(new char[2].AsMemory(), token);
            }
            catch (Exception e)
            {
                this.logger.LogWarning($"Get note failed: {e.Message}\n{e.StackTrace}");
                throw new OfflineException("Connection error during set note");
            }

            if (response == null)
            {
                throw new MumbleException("Did not receive note");
            }

            return response;
        }

        public async Task<string> GetHelp(CancellationToken token)
        {
            try
            {
                this.logger.LogDebug($"help");
                await this.writer!.WriteAsync($"help\n".AsMemory(), token);
                await this.writer.FlushAsync();
            }
            catch (Exception e)
            {
                this.logger.LogWarning($"Get help failed: {e.Message}\n{e.StackTrace}");
                throw new OfflineException("Connection error during help");
            }

            var sb = new StringBuilder();
            try
            {
                for (var i = 0; i < 10; i++)
                {
                    sb.AppendLine(await this.reader!.ReadLineAsync());
                }
            }
            catch (Exception e)
            {
                this.logger.LogWarning($"Get note failed: {e.Message}\n{e.StackTrace}");
                throw new OfflineException("Connection error during set note");
            }
            return sb.ToString();
        }

        public async Task WaitForUserInList(string username, CancellationToken token)
        {
            try
            {
                this.logger.LogDebug($"GetList");
                await this.writer!.WriteAsync($"user\n".AsMemory(), token);
                await this.writer.FlushAsync();
            }
            catch (Exception e)
            {
                this.logger.LogWarning($"Get help failed: {e.Message}\n{e.StackTrace}");
                throw new OfflineException("Connection error during help");
            }

            try
            {
                while (true)
                {
                    var line = await this.reader!.ReadLineAsync();
                    if (line != null && line.EndsWith(username))
                    {
                        return;
                    }
                }
            }
            catch (Exception e)
            {
                this.logger.LogWarning($"Get note failed: {e.Message}\n{e.StackTrace}");
                throw new OfflineException("Connection error during set note");
            }
        }

        public async Task WaitForNoteInList(string noteId, CancellationToken token)
        {
            try
            {
                this.logger.LogDebug($"GetList");
                await this.writer!.WriteAsync($"list\n".AsMemory(), token);
                await this.writer.FlushAsync();
            }
            catch (Exception e)
            {
                this.logger.LogWarning($"Get help failed: {e.Message}\n{e.StackTrace}");
                throw new OfflineException("Connection error during help");
            }

            try
            {
                while (true)
                {
                    var line = await this.reader!.ReadLineAsync();
                    if (line != null && line.EndsWith(noteId))
                    {
                        return;
                    }
                }
            }
            catch (Exception e)
            {
                this.logger.LogWarning($"Get note failed: {e.Message}\n{e.StackTrace}");
                throw new OfflineException("Connection error during set note");
            }
        }

        public void Dispose()
        {
            this.tcpClient?.Dispose();
            this.reader?.Dispose();
            this.writer?.Dispose();
        }
    }
}
