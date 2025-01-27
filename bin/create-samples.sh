#!/bin/zsh

# Define user data
users=(
    "aragorn|Aragorn Son of Arathorn|Strider|https://lotr.com/aragorn.jpg|aragorn@gondor.com|1975-03-01"
    "legolas|Legolas Thranduilion|Legolas|https://lotr.com/legolas.jpg|legolas@woodland.com|1980-07-19"
    "gimli|Gimli Son of Gloin|Gimli|https://lotr.com/gimli.jpg|gimli@erebor.com|1968-06-10"
    "frodo|Frodo Baggins|Frodo|https://lotr.com/frodo.jpg|frodo@shire.com|1985-09-22"
    "sam|Samwise Gamgee|Sam|https://lotr.com/sam.jpg|sam@shire.com|1983-04-06"
    "gandalf|Gandalf the Grey|Gandalf|https://lotr.com/gandalf.jpg|gandalf@valinor.com|1930-02-29"
    "sauron|Sauron the Deceiver|Sauron|https://lotr.com/sauron.jpg|sauron@barad-dur.com|1500-10-10"
    "luke|Luke Skywalker|Luke|https://starwars.com/luke.jpg|luke@rebellion.com|1977-05-04"
    "leia|Leia Organa|Leia|https://starwars.com/leia.jpg|leia@rebellion.com|1979-06-15"
    "han|Han Solo|Han|https://starwars.com/han.jpg|han@falcon.com|1974-08-12"
    "vader|Darth Vader|Vader|https://starwars.com/vader.jpg|vader@empire.com|1960-01-01"
    "palpatine|Emperor Palpatine|Palpatine|https://starwars.com/palpatine.jpg|palpatine@empire.com|1940-11-11"
    "obiwan|Obi-Wan Kenobi|Obi-Wan|https://starwars.com/obiwan.jpg|obiwan@jedi.com|1970-03-25"
    "yoda|Master Yoda|Yoda|https://starwars.com/yoda.jpg|yoda@jedi.com|900-01-01"
    "chewbacca|Chewbacca|Chewie|https://starwars.com/chewbacca.jpg|chewie@falcon.com|1950-10-20"
    "lando|Lando Calrissian|Lando|https://starwars.com/lando.jpg|lando@cloudcity.com|1972-07-07"
    "rey|Rey Skywalker|Rey|https://starwars.com/rey.jpg|rey@rebellion.com|2000-02-12"
    "kylo|Kylo Ren|Kylo|https://starwars.com/kylo.jpg|kylo@firstorder.com|1995-09-29"
    "maul|Darth Maul|Maul|https://starwars.com/maul.jpg|maul@empire.com|1975-12-15"
    "bilbo|Bilbo Baggins|Bilbo|https://lotr.com/bilbo.jpg|bilbo@shire.com|1958-09-22"
    "saruman|Saruman the White|Saruman|https://lotr.com/saruman.jpg|saruman@isengard.com|1910-01-14"
    "gollum|Sméagol|Gollum|https://lotr.com/gollum.jpg|gollum@cave.com|1800-06-06"
    "faramir|Faramir of Gondor|Faramir|https://lotr.com/faramir.jpg|faramir@gondor.com|1982-10-16"
    "boromir|Boromir of Gondor|Boromir|https://lotr.com/boromir.jpg|boromir@gondor.com|1978-02-20"
)

# Create users and profiles
user_id=3
declare -A user_map

for user in "${users[@]}"; do
    IFS='|' read -r username legal_name display_name profile_pic email dob <<< "$user"

    echo "Creating user: $username..."
    ./target/debug/enigma-client auth create "$username" "password123"

    echo "Creating profile for $username..."
    ./target/debug/enigma-client profiles create "$user_id" "$legal_name" "$display_name" "$profile_pic" "$email" "$dob"

    user_map[$username]=$user_id
    ((user_id++))
done

# Create social connections (friendships)
connections=(
    "aragorn legolas"
    "aragorn gimli"
    "frodo sam"
    "frodo gandalf"
    "luke leia"
    "luke han"
    "leia han"
    "vader palpatine"
    "obiwan yoda"
    "lando chewbacca"
    "rey kylo"
    "bilbo gollum"
    "boromir faramir"
    "gimli legolas"
    "gandalf saruman"
)

for connection in "${connections[@]}"; do
    IFS=' ' read -r user1 user2 <<< "$connection"

    id1=${user_map[$user1]}
    id2=${user_map[$user2]}

    echo "Creating connection request from $user1 ($id1) to $user2 ($id2)..."
    ./target/debug/enigma-client connections request "$id1" "$id2" friend

    echo "Accepting connection request for $user2 ($id2) to $user1 ($id1)..."
    ./target/debug/enigma-client connections accept "$id2" "$id1" friend
done

echo "Finished creating users and social connections!"
