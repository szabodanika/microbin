module MBP
  module BasicSyntaxHighlighter

		# Plugin Properties

    def self.get_id() 
      "BasicSyntaxHighlighter"
    end

    def self.get_name() 
      "Basic Syntax Highlighter Plugin"
    end

    def self.get_version() 
      "1.0.0"
    end

    def self.get_author() 
      "Daniel Szabo"
    end

    def self.get_webpage() 
      "https://github.com/szabodanika/microbin"
    end

    def self.get_description() 
      "This plugin will simply color keywords and special characters in four different colors based on some very basic RegEx - it is meant to univesally make code pastas more readable but is not a robust syntax highlighter solution."
    end

		# Plugin Event Hooks

    def self.init() 
      # Ignore event
      "OK"
    end
    
    def self.on_pasta_created(content)
      # We do not modify stored content
      return content
    end

    def self.on_pasta_read(content)

      tokens = {
          
          "orchid" => [/([0-9])/, /([t|T][r|R][u|U][e|E]|[f|F][a|A][l|L][s|S][e|E])/],

          "palevioletred" => ['(', ')', '{', '}', '[', ']'],

          "royalblue" => [/(\s(for|while|do|select|async|await|mut|break|continue|in|as|switch|let|fn|async|if|else|elseif|new|switch|match|case|default|public|protected|private|return|class|interface|static|final|const|var|int|integer|boolean|float|double|module|def|end|void))(?![a-z])/],

          "mediumorchid" => [/(:|\.|;|=|>|<|\?|!|#|%|@|\^|&|\*|\|)/],

          "mediumseagreen" => [/(\".*\")/, /(\'.*\')/]

        };

      tokens.each { | color, tokens | 
        for token in tokens do
          if(token.class == String) 
            content.gsub!(token, "$$#{color}$$" + token + "$$/#{color}$$")
          elsif
            content.gsub!(token, "$$#{color}$$" + '\1' + "$$/#{color}$$")
          end
        end
       };

      tokens.each { | color, tokens | 
        content.gsub!("$$#{color}$$", "<span style='color:#{color}'>");
        content.gsub!("$$/#{color}$$", "</span>");
      };

      return content

    end

  end
end