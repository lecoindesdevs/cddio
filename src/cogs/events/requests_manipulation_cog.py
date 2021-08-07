from discord.ext import commands
from discord.permissions import P
from datetime import datetime
import config

class RequestsThreads(commands.Cog):
    def __init__(self, bot):
        self.bot = bot
    
    """Basic on_message event which adds specific reactions."""
    @commands.Cog.listener()
    async def on_message(self, message):
        # If current channel isn't the requests channel
        if message.channel.id != config.CHANNELS['requests']:
            # Cancels.
            return
        
        # For each emoji
        for reaction in '✅🔐❌':
            # Adds emoji to message by using reactions.
            await message.add_reaction(reaction)

    """on_raw_reaction_add event which controls requests manipulations."""
    @commands.Cog.listener()
    async def on_raw_reaction_add(self, payload):
        # If it's my reaction
        if payload.user_id == self.bot.user.id:
            # Then cancels.
            return

        # If it's not in requests channel OR the reaction isn't a manipulation reaction
        if payload.channel_id != config.CHANNELS['requests'] or not str(payload.emoji) in '✅🔐❌':
            # Then cancels.
            return

        # Gets the channel by the payload's channel's ID. == Gets the channel by the requests channel ID.
        channel = self.bot.get_channel(payload.channel_id)
        # Gets the message by the payload's message's ID.
        message = await channel.fetch_message(payload.message_id)

        # Handling no mentions.
        try:
            # Gets request's author by finding his mention.
            customer = message.mentions[0]
        except:
            # If no mentions, then cancels.
            await message.remove_reaction(payload.emoji, payload.member)
            return

        # For each reaction in message's reactions
        for reaction in message.reactions:
            # If it's a lock
            if reaction.emoji == '🔒':
                # Then cancels.
                return
        
        # If the new reaction's emoji is a check mark
        if str(payload.emoji) == '✅':
            # Removes reaction (just prettier).
            await message.remove_reaction(payload.emoji, payload.member)

            # Checks if reaction's author has the developer role.
            is_developer = False

            # For each role in the reaction's author's roles
            for role in payload.member.roles:
                # If the role is the developer one
                if role.id == config.ROLES['developer']:
                    # Then validates that he's a developer.
                    is_developer = True

            # If he's not confirmed as developer
            if not is_developer:
                # Then cancels.
                return

            # Creates the new thread's name so it's unique.
            thread_name = f'{payload.member.display_name}>{message.id}>{datetime.now()}'

            # Creates the new thread with its previously created name and maximum auto_archive_duration.
            thread = await channel.start_thread(name=thread_name, auto_archive_duration=10080)

            # Adds the request's author to the new thread.
            await thread.add_user(customer)
            # Adds the reaction's author to the new thread.
            await thread.add_user(payload.member)

            # Gets the moderators' role.
            moderator_role = channel.guild.get_role(config.ROLES['moderator'])
            
            # For each moderator
            for moderator in moderator_role.members:
                # Adds the moderator to the thread.
                await thread.add_user(moderator)

        # If the new reaction's emoji is a lock with a key
        elif str(payload.emoji) == '🔐':
            # If the reaction's author isn't the request's author
            if payload.user_id != customer.id:
                # Then cancels.
                await message.remove_reaction(payload.emoji, payload.member)
                return
            
            # Clears all request's reactions.
            await message.clear_reactions()

            # For each thread in the guild's threads
            for thread in channel.guild.threads:
                # If the thread's name contains the request's ID
                if str(message.id) in thread.name:
                    # Then deletes the thread.
                    await thread.delete()

            # Adds the lock reaction to the request. == Locks the request.
            await message.add_reaction('🔒')
        
        #If the new reaction's emoji is a cross mark
        elif str(payload.emoji) == '❌':
            # Checks if reaction's author has the developer role.
            is_developer = False

            # For each role in the reaction's author's roles
            for role in payload.member.roles:
                # If the role is the developer one
                if role.id == config.ROLES['developer']:
                    # Then validates that he's a developer.
                    is_developer = True

            # If he's not confirmed as developer
            if not is_developer:
                # Then cancels.
                await message.remove_reaction(payload.emoji, payload.member)            
                return
            
            for reaction in message.reactions:
                if str(reaction.emoji) == '❌':
                    if reaction.count >= 5:
                        await message.delete()

def setup(bot):
    bot.add_cog(RequestsThreads(bot))
