from discord.ext import commands
import logging

class LogsEvents(commands.Cog):
    def __init__(self, bot):
        self.bot = bot

    @commands.Cog.listener()
    async def on_command(self, ctx):
        logging.info(f'{ctx.author} ({ctx.author.id}) has triggered the `{ctx.command.name}` ({ctx.command.cog_name}) command in the {ctx.channel.name} ({ctx.channel.id}) channel.')

    @commands.Cog.listener()
    async def on_command_error(self, ctx, error):
        logging.error(f'{ctx.author} ({ctx.author.id}) has raised an error in the {ctx.channel.name} ({ctx.channel.id}) channel:\n{error}')

def setup(bot):
    bot.add_cog(LogsEvents(bot))
