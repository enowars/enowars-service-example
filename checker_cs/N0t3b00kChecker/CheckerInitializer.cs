using EnoCore.Checker;
using Microsoft.Extensions.DependencyInjection;
using N0t3b00kChecker.Db;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace N0t3b00kChecker
{
    public class CheckerInitializer : ICheckerInitializer
    {
        public int FlagVariants => 1;

        public int NoiseVariants => 1;

        public int HavocVariants => 3;

        public string ServiceName => "n0t3b00k";

        public void Initialize(IServiceCollection collection)
        {
            collection.AddSingleton(typeof(CheckerDb));
            collection.AddTransient(typeof(NotebookClient));
        }
    }
}
