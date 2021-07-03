from discord.ext import commands
import logging

class LogsEvents(commands.Cog):
    def __init__(self, bot):
        self.bot = bot

    """Basic on_ready event which runs when bot is set."""
    @commands.Cog.listener()
    async def on_ready(self):
        logging.info(f'I\'ve logged in as {self.bot.user}.')

    """Basic command logger."""
    @commands.Cog.listener()
    async def on_command(self, ctx):
        logging.info(f'{ctx.author} ({ctx.author.id}) has triggered the `{ctx.command.name}` ({ctx.command.cog_name}) command in the {ctx.channel.name} ({ctx.channel.id}) channel.')

    """Basic command completion logger."""
    @commands.Cog.listener()
    async def on_command_completion(self, ctx):
        logging.info(f'{ctx.author} ({ctx.author.id}) completed the `{ctx.command.name}` ({ctx.command.cog_name}) command in the {ctx.channel.name} ({ctx.channel.id}) channel.')

    """Basic command error logger."""
    @commands.Cog.listener()
    async def on_command_error(self, ctx, error):
        logging.error(f'{ctx.author} ({ctx.author.id}) has raised an error in the {ctx.channel.name} ({ctx.channel.id}) channel:\n{error}')

def setup(bot):
    bot.add_cog(LogsEvents(bot))
