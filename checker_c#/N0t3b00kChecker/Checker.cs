using Bogus;
using EnoCore.Checker;
using EnoCore.Models;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using N0t3b00kChecker.Db;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading;
using System.Threading.Tasks;

namespace N0t3b00kChecker
{
    public class Checker : IChecker
    {
        public static int SERVICE_PORT = 2323;
        private static readonly string HELP = "\nThis is a notebook service. Commands:\n" +
            "reg USER PW - Register new account\n" +
            "log USER PW - Login to account\n" +
            "set TEXT..... - Set a note\n" +
            "user  - List all users\n" +
            "list - List all notes\n" +
            "exit - Exit!\n" +
            "dump - Dump the database\n" +
            "get ID";
        private readonly static Faker faker = new Faker("de");
        private readonly IServiceProvider serviceProvider;
        private readonly ILogger<Checker> logger;
        private readonly CheckerDb checkerDb;
        

        public Checker(ILogger<Checker> logger, CheckerDb checkerDb, IServiceProvider serviceProvider)
        {
            this.logger = logger;
            this.checkerDb = checkerDb;
            this.serviceProvider = serviceProvider;
        }

        public async Task HandleGetFlag(CheckerTaskMessage task, CancellationToken token)
        {
            switch (task.VariantId)
            {
                case 0:
                    await CheckFlagInNote(task, token);
                    break;
                default:
                    throw new InvalidOperationException();
            }
        }

        public async Task HandleGetNoise(CheckerTaskMessage task, CancellationToken token)
        {
            switch (task.VariantId)
            {
                case 0:
                    await CheckNoiseInNote(task, token);
                    break;
                default:
                    throw new InvalidOperationException();
            }
        }

        public async Task HandleHavoc(CheckerTaskMessage task, CancellationToken token)
        {
            switch (task.VariantId)
            {
                case 0:
                    await HavocHelp(task, token);
                    break;
                case 1:
                    await HavocUser(task, token);
                    break;
                case 2:
                    await HavocNotesList(task, token);
                    break;
                default:
                    throw new InvalidOperationException();
            }
        }

        public async Task HandlePutFlag(CheckerTaskMessage task, CancellationToken token)
        {
            switch (task.VariantId)
            {
                case 0:
                    await DeployFlagToNote(task, token);
                    break;
                default:
                    throw new InvalidOperationException();
            }
        }

        public async Task HandlePutNoise(CheckerTaskMessage task, CancellationToken token)
        {
            switch (task.VariantId)
            {
                case 0:
                    await DeployNoiseToNote(task, token);
                    break;
                default:
                    throw new InvalidOperationException();
            }
        }

        private async Task DeployFlagToNote(CheckerTaskMessage task, CancellationToken token)
        {
            using var client = this.serviceProvider.GetRequiredService<NotebookClient>();

            // Connect
            await client.Connect(task, token);

            // Register
            var user = new NotebookUser()
            {
                Username = faker.Internet.UserName() + DateTime.UtcNow.Millisecond,
                Password = faker.Internet.Password(),
                Note = task.Flag,
                TaskChainId = task.TaskChainId
            };
            await client.Register(user, token);
            await client.Login(user, token);

            // Deploy flag to note
            user.NoteId = await client.SetNote(task.Flag!, token);

            // Save creds
            await checkerDb.InsertNotebookUser(user, token);

            // Exit TODO
        }

        private async Task CheckFlagInNote(CheckerTaskMessage task, CancellationToken token)
        {
            using var client = this.serviceProvider.GetRequiredService<NotebookClient>();
            var user = await this.checkerDb.FindNotebookUser(task.TaskChainId, token);
            await client.Connect(task, token);

            // Login
            await client.Login(user, token);

            var note = await client.GetNote(user.NoteId!, token);

            if (note != task.Flag)
            {
                this.logger.LogWarning($"Expected flag, got {note}");
                throw new MumbleException($"Flag is no longer in note");
            }

            // Exit TODO
        }

        private async Task DeployNoiseToNote(CheckerTaskMessage task, CancellationToken token)
        {
            using var client = this.serviceProvider.GetRequiredService<NotebookClient>();

            // Connect
            await client.Connect(task, token);

            // Register
            var user = new NotebookUser()
            {
                Username = faker.Internet.UserName() + DateTime.UtcNow.Millisecond,
                Password = faker.Internet.Password(),
                Note = faker.Company.Bs(),
                TaskChainId = task.TaskChainId
            };
            await client.Register(user, token);
            await client.Login(user, token);

            // Deploy flag to note
            user.NoteId = await client.SetNote(user.Note, token);

            // Save creds
            await checkerDb.InsertNotebookUser(user, token);

            // Exit TODO
        }


        private async Task CheckNoiseInNote(CheckerTaskMessage task, CancellationToken token)
        {
            using var client = this.serviceProvider.GetRequiredService<NotebookClient>();
            var user = await this.checkerDb.FindNotebookUser(task.TaskChainId, token);
            await client.Connect(task, token);

            // Login
            await client.Login(user, token);

            var note = await client.GetNote(user.NoteId!, token);

            if (note != user.Note)
            {
                this.logger.LogWarning($"Expected flag, got {note}");
                throw new MumbleException($"Flag is no longer in note");
            }

            // Exit TODO
        }

        private async Task HavocHelp(CheckerTaskMessage task, CancellationToken token)
        {
            this.logger.LogInformation("Testing the help menu");
            using var client = this.serviceProvider.GetRequiredService<NotebookClient>();
            await client.Connect(task, token);
            
            var help = await client.GetHelp(token);
            if (help != HELP)
            {
                this.logger.LogInformation($"Unexpected help menu:{BitConverter.ToString(Encoding.Default.GetBytes(help))}\nvs\n{BitConverter.ToString(Encoding.Default.GetBytes(HELP))}");
                throw new MumbleException("Could not get help menu");
            }
        }

        private async Task HavocUser(CheckerTaskMessage task, CancellationToken token)
        {
            this.logger.LogInformation("Testing the user list");
            using var client = this.serviceProvider.GetRequiredService<NotebookClient>();
            await client.Connect(task, token);

            var user = new NotebookUser()
            {
                Username = faker.Internet.UserName() + DateTime.UtcNow.Millisecond,
                Password = faker.Internet.Password(),
                Note = null,
                TaskChainId = task.TaskChainId
            };
            await client.Register(user, token);

            var foundUsers = await client.GetUsers(token);
            foreach (var foundUser in foundUsers)
            {
                if (user.Username == foundUser)
                {
                    return;
                }
            }

            throw new MumbleException("User missing from user list");
        }

        private async Task HavocNotesList(CheckerTaskMessage task, CancellationToken token)
        {
            using var client = this.serviceProvider.GetRequiredService<NotebookClient>();

            // Connect
            await client.Connect(task, token);

            // Register
            var user = new NotebookUser()
            {
                Username = faker.Internet.UserName() + DateTime.UtcNow.Millisecond,
                Password = faker.Internet.Password(),
                Note = faker.Company.Bs(),
                TaskChainId = task.TaskChainId
            };
            await client.Register(user, token);
            await client.Login(user, token);

            // Deploy flag to note
            user.NoteId = await client.SetNote(user.Note, token);

            var foundNotes = await client.GetNotes(token);
            foreach (var foundNote in foundNotes)
            {
                if (foundNote == user.NoteId)
                {
                    return;
                }
            }

            throw new MumbleException("Note missing from user list");
        }
    }
}
