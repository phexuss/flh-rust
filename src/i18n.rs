pub fn get_text<'a>(lang: &str, key: &'a str) -> &'a str {
    let lang_code = match lang.to_lowercase().as_str() {
        "uk" | "ua" => "uk",
        _ => "ru",
    };

    match (lang_code, key) {
        // Main & Bot status
        ("ru", "bot_started_admin") => "🟢 <b>Freelance Sniper Bot запущен</b>\nИнтервал парсинга: {interval} мин.",
        ("uk", "bot_started_admin") => "🟢 <b>Freelance Sniper Bot запущено</b>\nІнтервал парсингу: {interval} хв.",

        ("ru", "start_text") => "<b>Freelance Sniper Bot</b>\n\nЯ мониторю новые проекты на FreelanceHunt и присылаю тебе уведомления.\n\n<b>Категории:</b>\n• HTML/CSS верстка\n• JavaScript / TypeScript\n• Python\n• Веб-программирование\n• Криптовалюта и blockchain\n• Парсинг данных\n• Разработка ботов\n\n<b>Команды:</b>\n/start — это сообщение\n/help — справка\n/status — статус бота\n",
        ("uk", "start_text") => "<b>Freelance Sniper Bot</b>\n\nЯ моніторю нові проєкти на FreelanceHunt і надсилаю тобі сповіщення.\n\n<b>Категорії:</b>\n• HTML/CSS верстка\n• JavaScript / TypeScript\n• Python\n• Веб-програмування\n• Криптовалюта та blockchain\n• Парсинг даних\n• Розробка ботів\n\n<b>Команди:</b>\n/start — це повідомлення\n/help — довідка\n/status — статус бота\n",

        ("ru", "help_text") => "<b>Справка</b>\n\nБот автоматически проверяет новые проекты каждые несколько минут.\n\nДля каждого нового проекта ты получишь:\n• Описание, бюджет, навыки\n• Кнопку для открытия проекта\n• Кнопку генерации отклика (AI)\n• Кнопку анализа проекта (AI)\n\n/status — текущий статус и кол-во обработанных проектов",
        ("uk", "help_text") => "<b>Довідка</b>\n\nБот автоматично перевіряє нові проєкти кожні кілька хвилин.\n\nДля кожного нового проєкту ти отримаєш:\n• Опис, бюджет, навички\n• Кнопку для відкриття проєкту\n• Кнопку генерації відгуку (AI)\n• Кнопку аналізу проєкту (AI)\n\n/status — поточний статус і кількість оброблених проєктів",

        ("ru", "status_text") => "📊 <b>Статус Freelance Sniper</b>\n\n✅ Состояние: <b>Работает</b>\n⏱ Аптайм: <b>{uptime}</b>\n🔄 Последний парсинг: <b>{last_parse}</b>\n📁 В базе: <b>{count}</b> проектов\n",
        ("uk", "status_text") => "📊 <b>Статус Freelance Sniper</b>\n\n✅ Стан: <b>Працює</b>\n⏱ Аптайм: <b>{uptime}</b>\n🔄 Останній парсинг: <b>{last_parse}</b>\n📁 У базі: <b>{count}</b> проєктів\n",

        ("ru", "status_never") => "еще не был",
        ("uk", "status_never") => "ще не був",

        // Notifier & buttons
        ("ru", "notify_new_project") => "📌 <b>Новый проект на FreelanceHunt</b>",
        ("uk", "notify_new_project") => "📌 <b>Новий проєкт на FreelanceHunt</b>",

        ("ru", "notify_title") => "📝 <b>Название:</b> {name}",
        ("uk", "notify_title") => "📝 <b>Назва:</b> {name}",

        ("ru", "notify_budget") => "💰 <b>Бюджет:</b> {budget}",
        ("uk", "notify_budget") => "💰 <b>Бюджет:</b> {budget}",

        ("ru", "notify_skills") => "🏷 <b>Навыки:</b> {skills}",
        ("uk", "notify_skills") => "🏷 <b>Навички:</b> {skills}",

        ("ru", "notify_employer") => "👤 <b>Заказчик:</b> {employer}",
        ("uk", "notify_employer") => "👤 <b>Замовник:</b> {employer}",

        ("ru", "notify_published") => "📅 <b>Опубликовано:</b> {published}",
        ("uk", "notify_published") => "📅 <b>Опубліковано:</b> {published}",

        ("ru", "notify_bids") => "📊 <b>Откликов:</b> {bids}",
        ("uk", "notify_bids") => "📊 <b>Відгуків:</b> {bids}",

        ("ru", "notify_desc") => "📄 <b>Описание:</b>\n",
        ("uk", "notify_desc") => "📄 <b>Опис:</b>\n",

        ("ru", "btn_open_project") => "🔗 Открыть проект",
        ("uk", "btn_open_project") => "🔗 Відкрити проєкт",

        ("ru", "btn_gen_bid") => "🤖 Сгенерировать отклика",
        ("uk", "btn_gen_bid") => "🤖 Сгенерувати відгук",

        ("ru", "btn_analyze") => "📊 Анализ",
        ("uk", "btn_analyze") => "📊 Аналіз",

        ("ru", "btn_regen") => "🔄 Перегенерировать",
        ("uk", "btn_regen") => "🔄 Перегенерувати",

        ("ru", "btn_edit") => "✏️ Редактировать",
        ("uk", "btn_edit") => "✏️ Редагувати",

        ("ru", "btn_send_bid") => "✅ Отправить на сайт",
        ("uk", "btn_send_bid") => "✅ Відправити на сайт",

        ("ru", "btn_cancel") => "❌ Отмена",
        ("uk", "btn_cancel") => "❌ Скасувати",

        ("ru", "analyze_loading") => "⏳ Анализирую проект...",
        ("uk", "analyze_loading") => "⏳ Аналізую проєкт...",

        ("ru", "bid_enter_budget") => "💵 Введите бюджет и срок через пробел (например: 3000 5)\nИли /cancel для отмены",
        ("uk", "bid_enter_budget") => "💵 Введіть бюджет і термін через пробіл (наприклад: 3000 5)\nАбо /cancel для скасування",

        ("ru", "bid_format_error") => "❌ Ошибка формата. Пожалуйста, введите ДВА числа через пробел (бюджет и дни), например: 3000 5",
        ("uk", "bid_format_error") => "❌ Помилка формату. Будь ласка, введіть ДВА числа через пробіл (бюджет і дні), наприклад: 3000 5",

        ("ru", "bid_generating") => "⏳ Генерирую отклик...",
        ("uk", "bid_generating") => "⏳ Генерую відгук...",

        ("ru", "bid_sending") => "⏳ Отправка отклика на сайт...",
        ("uk", "bid_sending") => "⏳ Відправка відгуку на сайт...",

        ("ru", "bid_success") => "✅ Отклик успешно отправлен на сайт!",
        ("uk", "bid_success") => "✅ Відгук успішно відправлено на сайт!",

        ("ru", "bid_canceled") => "Отклик отменен.",
        ("uk", "bid_canceled") => "Відгук скасовано.",

        ("ru", "notify_unknown") => "Неизвестно",
        ("uk", "notify_unknown") => "Невідомо",

        _ => key,
    }
}
