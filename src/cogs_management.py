import os

def add_cogs(bot, cogs_dir):
    # For each file in the cogs directory:
    for filename in os.listdir(cogs_dir):
        # If file's name ends with ".py":
        if filename.endswith('.py'):
            # Processes the path.
            extension_path = cogs_dir.replace('./', '').replace('/', '.')

            bot.load_extension(f'{extension_path}.{filename[:-3]}')

        # If it's a directory:
        if os.path.isdir(f'{cogs_dir}/{filename}'):
            # Recursivity omg
            add_cogs(bot, f'{cogs_dir}/{filename}')
