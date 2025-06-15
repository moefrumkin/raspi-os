def Setting(**kwargs):
    if kwargs['language'] == 'rust':
        return {
                'ls': {
                    'rust': {
                        'features': ['raspi3']
                        }
                    }
                }
