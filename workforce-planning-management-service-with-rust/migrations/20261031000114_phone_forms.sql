-- Migration: a telephone number kept **three ways** (WPM-D152 question 2, WPM-T382): as typed (unchanged), as ASCII
-- digits for search and export, and in E.164 where it can be derived. Adds the derived columns to
-- `worker_contact_details` (the mobile and home numbers) and `emergency_contacts` (the number and the alternate), and
-- backfills existing rows.
--
-- The backfill is SQL, so this crate stays independent of the service. It follows `rules::phone` in the service: the
-- digits of any decimal digit set the service accepts become ASCII; a number written with `+` is its own E.164; a
-- worker's number written without `+` is read in the address's country, else the country of `work_jurisdiction`, when
-- that country is in [`CALLING_CODES`] (its trunk prefix dropped; the North American plan takes ten digits, or eleven
-- starting `1`); a number starting `00`, a country not in the table, or a result over 15 digits gets no E.164 form. An
-- emergency contact's number is not read in the worker's country (the contact may live elsewhere): only `+` gives it
-- one. The tables here are copies of the service's, frozen with this migration; a service test checks they are equal
-- and that this SQL and the rule agree on the same rows.

ALTER TABLE worker_contact_details
                 ADD COLUMN IF NOT EXISTS phone_mobile_digits VARCHAR(15) NULL,
                 ADD COLUMN IF NOT EXISTS phone_mobile_e164 VARCHAR(16) NULL,
                 ADD COLUMN IF NOT EXISTS phone_home_digits VARCHAR(15) NULL,
                 ADD COLUMN IF NOT EXISTS phone_home_e164 VARCHAR(16) NULL;

ALTER TABLE emergency_contacts
                 ADD COLUMN IF NOT EXISTS phone_digits VARCHAR(15) NULL,
                 ADD COLUMN IF NOT EXISTS phone_e164 VARCHAR(16) NULL,
                 ADD COLUMN IF NOT EXISTS alt_phone_digits VARCHAR(15) NULL,
                 ADD COLUMN IF NOT EXISTS alt_phone_e164 VARCHAR(16) NULL;

UPDATE worker_contact_details SET phone_mobile_digits = NULLIF(regexp_replace(translate(phone_mobile, '０１２３４５６７８９٠١٢٣٤٥٦٧٨٩۰۱۲۳۴۵۶۷۸۹߀߁߂߃߄߅߆߇߈߉०१२३४५६७८९০১২৩৪৫৬৭৮৯੦੧੨੩੪੫੬੭੮੯૦૧૨૩૪૫૬૭૮૯୦୧୨୩୪୫୬୭୮୯௦௧௨௩௪௫௬௭௮௯౦౧౨౩౪౫౬౭౮౯೦೧೨೩೪೫೬೭೮೯൦൧൨൩൪൫൬൭൮൯෦෧෨෩෪෫෬෭෮෯๐๑๒๓๔๕๖๗๘๙໐໑໒໓໔໕໖໗໘໙༠༡༢༣༤༥༦༧༨༩၀၁၂၃၄၅၆၇၈၉႐႑႒႓႔႕႖႗႘႙០១២៣៤៥៦៧៨៩᠐᠑᠒᠓᠔᠕᠖᠗᠘᠙᥆᥇᥈᥉᥊᥋᥌᥍᥎᥏᧐᧑᧒᧓᧔᧕᧖᧗᧘᧙᭐᭑᭒᭓᭔᭕᭖᭗᭘᭙᮰᮱᮲᮳᮴᮵᮶᮷᮸᮹᱀᱁᱂᱃᱄᱅᱆᱇᱈᱉᱐᱑᱒᱓᱔᱕᱖᱗᱘᱙꘠꘡꘢꘣꘤꘥꘦꘧꘨꘩꧐꧑꧒꧓꧔꧕꧖꧗꧘꧙꧰꧱꧲꧳꧴꧵꧶꧷꧸꧹꩐꩑꩒꩓꩔꩕꩖꩗꩘꩙꯰꯱꯲꯳꯴꯵꯶꯷꯸꯹', '01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789'), '[^0-9]', '', 'g'), ''), phone_home_digits = NULLIF(regexp_replace(translate(phone_home, '０１２３４５６７８９٠١٢٣٤٥٦٧٨٩۰۱۲۳۴۵۶۷۸۹߀߁߂߃߄߅߆߇߈߉०१२३४५६७८९০১২৩৪৫৬৭৮৯੦੧੨੩੪੫੬੭੮੯૦૧૨૩૪૫૬૭૮૯୦୧୨୩୪୫୬୭୮୯௦௧௨௩௪௫௬௭௮௯౦౧౨౩౪౫౬౭౮౯೦೧೨೩೪೫೬೭೮೯൦൧൨൩൪൫൬൭൮൯෦෧෨෩෪෫෬෭෮෯๐๑๒๓๔๕๖๗๘๙໐໑໒໓໔໕໖໗໘໙༠༡༢༣༤༥༦༧༨༩၀၁၂၃၄၅၆၇၈၉႐႑႒႓႔႕႖႗႘႙០១២៣៤៥៦៧៨៩᠐᠑᠒᠓᠔᠕᠖᠗᠘᠙᥆᥇᥈᥉᥊᥋᥌᥍᥎᥏᧐᧑᧒᧓᧔᧕᧖᧗᧘᧙᭐᭑᭒᭓᭔᭕᭖᭗᭘᭙᮰᮱᮲᮳᮴᮵᮶᮷᮸᮹᱀᱁᱂᱃᱄᱅᱆᱇᱈᱉᱐᱑᱒᱓᱔᱕᱖᱗᱘᱙꘠꘡꘢꘣꘤꘥꘦꘧꘨꘩꧐꧑꧒꧓꧔꧕꧖꧗꧘꧙꧰꧱꧲꧳꧴꧵꧶꧷꧸꧹꩐꩑꩒꩓꩔꩕꩖꩗꩘꩙꯰꯱꯲꯳꯴꯵꯶꯷꯸꯹', '01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789'), '[^0-9]', '', 'g'), '');

UPDATE emergency_contacts SET phone_digits = NULLIF(regexp_replace(translate(phone, '０１２３４５６７８９٠١٢٣٤٥٦٧٨٩۰۱۲۳۴۵۶۷۸۹߀߁߂߃߄߅߆߇߈߉०१२३४५६७८९০১২৩৪৫৬৭৮৯੦੧੨੩੪੫੬੭੮੯૦૧૨૩૪૫૬૭૮૯୦୧୨୩୪୫୬୭୮୯௦௧௨௩௪௫௬௭௮௯౦౧౨౩౪౫౬౭౮౯೦೧೨೩೪೫೬೭೮೯൦൧൨൩൪൫൬൭൮൯෦෧෨෩෪෫෬෭෮෯๐๑๒๓๔๕๖๗๘๙໐໑໒໓໔໕໖໗໘໙༠༡༢༣༤༥༦༧༨༩၀၁၂၃၄၅၆၇၈၉႐႑႒႓႔႕႖႗႘႙០១២៣៤៥៦៧៨៩᠐᠑᠒᠓᠔᠕᠖᠗᠘᠙᥆᥇᥈᥉᥊᥋᥌᥍᥎᥏᧐᧑᧒᧓᧔᧕᧖᧗᧘᧙᭐᭑᭒᭓᭔᭕᭖᭗᭘᭙᮰᮱᮲᮳᮴᮵᮶᮷᮸᮹᱀᱁᱂᱃᱄᱅᱆᱇᱈᱉᱐᱑᱒᱓᱔᱕᱖᱗᱘᱙꘠꘡꘢꘣꘤꘥꘦꘧꘨꘩꧐꧑꧒꧓꧔꧕꧖꧗꧘꧙꧰꧱꧲꧳꧴꧵꧶꧷꧸꧹꩐꩑꩒꩓꩔꩕꩖꩗꩘꩙꯰꯱꯲꯳꯴꯵꯶꯷꯸꯹', '01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789'), '[^0-9]', '', 'g'), ''), alt_phone_digits = NULLIF(regexp_replace(translate(alt_phone, '０１２３４５６７８９٠١٢٣٤٥٦٧٨٩۰۱۲۳۴۵۶۷۸۹߀߁߂߃߄߅߆߇߈߉०१२३४५६७८९০১২৩৪৫৬৭৮৯੦੧੨੩੪੫੬੭੮੯૦૧૨૩૪૫૬૭૮૯୦୧୨୩୪୫୬୭୮୯௦௧௨௩௪௫௬௭௮௯౦౧౨౩౪౫౬౭౮౯೦೧೨೩೪೫೬೭೮೯൦൧൨൩൪൫൬൭൮൯෦෧෨෩෪෫෬෭෮෯๐๑๒๓๔๕๖๗๘๙໐໑໒໓໔໕໖໗໘໙༠༡༢༣༤༥༦༧༨༩၀၁၂၃၄၅၆၇၈၉႐႑႒႓႔႕႖႗႘႙០១២៣៤៥៦៧៨៩᠐᠑᠒᠓᠔᠕᠖᠗᠘᠙᥆᥇᥈᥉᥊᥋᥌᥍᥎᥏᧐᧑᧒᧓᧔᧕᧖᧗᧘᧙᭐᭑᭒᭓᭔᭕᭖᭗᭘᭙᮰᮱᮲᮳᮴᮵᮶᮷᮸᮹᱀᱁᱂᱃᱄᱅᱆᱇᱈᱉᱐᱑᱒᱓᱔᱕᱖᱗᱘᱙꘠꘡꘢꘣꘤꘥꘦꘧꘨꘩꧐꧑꧒꧓꧔꧕꧖꧗꧘꧙꧰꧱꧲꧳꧴꧵꧶꧷꧸꧹꩐꩑꩒꩓꩔꩕꩖꩗꩘꩙꯰꯱꯲꯳꯴꯵꯶꯷꯸꯹', '01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789'), '[^0-9]', '', 'g'), '');

WITH plans(country, code, trunk) AS (VALUES ('AD', '376', NULL), ('AE', '971', '0'), ('AR', '54', '0'), ('AT', '43', '0'), ('AU', '61', '0'), ('BD', '880', '0'), ('BE', '32', '0'), ('BG', '359', '0'), ('BH', '973', NULL), ('BR', '55', '0'), ('BY', '375', '8'), ('BZ', '501', NULL), ('CA', '1', NULL), ('CH', '41', '0'), ('CN', '86', '0'), ('CR', '506', NULL), ('CY', '357', NULL), ('CZ', '420', NULL), ('DE', '49', '0'), ('DK', '45', NULL), ('DZ', '213', '0'), ('EE', '372', NULL), ('EG', '20', '0'), ('ES', '34', NULL), ('ET', '251', '0'), ('FI', '358', '0'), ('FR', '33', '0'), ('GB', '44', '0'), ('GH', '233', '0'), ('GR', '30', NULL), ('GT', '502', NULL), ('HK', '852', NULL), ('HN', '504', NULL), ('HR', '385', '0'), ('HU', '36', '06'), ('ID', '62', '0'), ('IE', '353', '0'), ('IL', '972', '0'), ('IN', '91', '0'), ('IR', '98', '0'), ('IS', '354', NULL), ('IT', '39', NULL), ('JO', '962', '0'), ('JP', '81', '0'), ('KE', '254', '0'), ('KR', '82', '0'), ('KW', '965', NULL), ('KZ', '7', '8'), ('LB', '961', '0'), ('LK', '94', '0'), ('LU', '352', NULL), ('LV', '371', NULL), ('MA', '212', '0'), ('MC', '377', NULL), ('MO', '853', NULL), ('MT', '356', NULL), ('MX', '52', NULL), ('MY', '60', '0'), ('NG', '234', '0'), ('NI', '505', NULL), ('NL', '31', '0'), ('NO', '47', NULL), ('NZ', '64', '0'), ('OM', '968', NULL), ('PA', '507', NULL), ('PE', '51', '0'), ('PH', '63', '0'), ('PK', '92', '0'), ('PL', '48', NULL), ('PT', '351', NULL), ('QA', '974', NULL), ('RO', '40', '0'), ('RS', '381', '0'), ('RU', '7', '8'), ('SA', '966', '0'), ('SE', '46', '0'), ('SG', '65', NULL), ('SI', '386', '0'), ('SK', '421', '0'), ('SM', '378', NULL), ('SV', '503', NULL), ('TH', '66', '0'), ('TN', '216', '0'), ('TR', '90', '0'), ('TW', '886', '0'), ('TZ', '255', '0'), ('UA', '380', '0'), ('UG', '256', '0'), ('US', '1', NULL), ('VN', '84', '0'), ('ZA', '27', '0')),
                  src AS (
                      SELECT d.id,
                             COALESCE(d.country, CASE WHEN split_part(w.work_jurisdiction, '-', 1) ~ '^[A-Z]{2}$'
                                                      THEN split_part(w.work_jurisdiction, '-', 1) END) AS country
                        FROM worker_contact_details d LEFT JOIN workers w ON w.pid = d.worker_pid)
             UPDATE worker_contact_details d
                SET phone_mobile_e164 = CASE
           WHEN d.phone_mobile_digits IS NULL THEN NULL
           WHEN ltrim(translate(d.phone_mobile, '＋　', '+ ')) LIKE '+%'
                THEN CASE WHEN d.phone_mobile_digits !~ '^0' AND length(d.phone_mobile_digits) <= 15 THEN '+' || d.phone_mobile_digits END
           WHEN p.code IS NULL OR d.phone_mobile_digits LIKE '00%' THEN NULL
           WHEN p.code = '1' THEN CASE WHEN d.phone_mobile_digits ~ '^[2-9][0-9]{9}$' THEN '+1' || d.phone_mobile_digits
                                       WHEN d.phone_mobile_digits ~ '^1[2-9][0-9]{9}$' THEN '+' || d.phone_mobile_digits END
           ELSE (SELECT CASE WHEN length(n) >= 4 AND length(p.code) + length(n) <= 15 THEN '+' || p.code || n END
                   FROM (SELECT CASE WHEN p.trunk IS NOT NULL AND left(d.phone_mobile_digits, length(p.trunk)) = p.trunk
                                     THEN substr(d.phone_mobile_digits, length(p.trunk) + 1) ELSE d.phone_mobile_digits END AS n) national)
         END, phone_home_e164 = CASE
           WHEN d.phone_home_digits IS NULL THEN NULL
           WHEN ltrim(translate(d.phone_home, '＋　', '+ ')) LIKE '+%'
                THEN CASE WHEN d.phone_home_digits !~ '^0' AND length(d.phone_home_digits) <= 15 THEN '+' || d.phone_home_digits END
           WHEN p.code IS NULL OR d.phone_home_digits LIKE '00%' THEN NULL
           WHEN p.code = '1' THEN CASE WHEN d.phone_home_digits ~ '^[2-9][0-9]{9}$' THEN '+1' || d.phone_home_digits
                                       WHEN d.phone_home_digits ~ '^1[2-9][0-9]{9}$' THEN '+' || d.phone_home_digits END
           ELSE (SELECT CASE WHEN length(n) >= 4 AND length(p.code) + length(n) <= 15 THEN '+' || p.code || n END
                   FROM (SELECT CASE WHEN p.trunk IS NOT NULL AND left(d.phone_home_digits, length(p.trunk)) = p.trunk
                                     THEN substr(d.phone_home_digits, length(p.trunk) + 1) ELSE d.phone_home_digits END AS n) national)
         END
               FROM src LEFT JOIN plans p ON p.country = src.country
              WHERE src.id = d.id;

UPDATE emergency_contacts c SET phone_e164 = CASE
           WHEN c.phone_digits IS NULL THEN NULL
           WHEN ltrim(translate(c.phone, '＋　', '+ ')) LIKE '+%'
                THEN CASE WHEN c.phone_digits !~ '^0' AND length(c.phone_digits) <= 15 THEN '+' || c.phone_digits END
           WHEN p.code IS NULL OR c.phone_digits LIKE '00%' THEN NULL
           WHEN p.code = '1' THEN CASE WHEN c.phone_digits ~ '^[2-9][0-9]{9}$' THEN '+1' || c.phone_digits
                                       WHEN c.phone_digits ~ '^1[2-9][0-9]{9}$' THEN '+' || c.phone_digits END
           ELSE (SELECT CASE WHEN length(n) >= 4 AND length(p.code) + length(n) <= 15 THEN '+' || p.code || n END
                   FROM (SELECT CASE WHEN p.trunk IS NOT NULL AND left(c.phone_digits, length(p.trunk)) = p.trunk
                                     THEN substr(c.phone_digits, length(p.trunk) + 1) ELSE c.phone_digits END AS n) national)
         END, alt_phone_e164 = CASE
           WHEN c.alt_phone_digits IS NULL THEN NULL
           WHEN ltrim(translate(c.alt_phone, '＋　', '+ ')) LIKE '+%'
                THEN CASE WHEN c.alt_phone_digits !~ '^0' AND length(c.alt_phone_digits) <= 15 THEN '+' || c.alt_phone_digits END
           WHEN p.code IS NULL OR c.alt_phone_digits LIKE '00%' THEN NULL
           WHEN p.code = '1' THEN CASE WHEN c.alt_phone_digits ~ '^[2-9][0-9]{9}$' THEN '+1' || c.alt_phone_digits
                                       WHEN c.alt_phone_digits ~ '^1[2-9][0-9]{9}$' THEN '+' || c.alt_phone_digits END
           ELSE (SELECT CASE WHEN length(n) >= 4 AND length(p.code) + length(n) <= 15 THEN '+' || p.code || n END
                   FROM (SELECT CASE WHEN p.trunk IS NOT NULL AND left(c.alt_phone_digits, length(p.trunk)) = p.trunk
                                     THEN substr(c.alt_phone_digits, length(p.trunk) + 1) ELSE c.alt_phone_digits END AS n) national)
         END
               FROM (SELECT NULL::text AS code, NULL::text AS trunk) p;
