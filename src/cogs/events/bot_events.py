from discord.ext import commands

class BotEvents(commands.Cog):
    def __init__(self, bot):
        self.bot = bot

    @commands.Cog.listener()
    async def on_ready(self):
        print(f'I\'ve logged in as {self.bot.user}.')

def setup(bot):
    bot.add_cog(BotEvents(bot))
