#!/usr/bin/env python3
"""
Analyze Whisper workflow log to estimate OpenAI API costs.
"""

import re
from datetime import datetime
from collections import defaultdict

def parse_log_file(filepath):
    """
    Parse the log file and extract recordings by date.

    Returns:
        dict: {date: [{'time': str, 'text': str}, ...]}
    """
    logs_by_date = defaultdict(list)
    current_date = None
    current_time = None

    with open(filepath, 'r') as f:
        lines = f.readlines()

    i = 0
    while i < len(lines):
        line = lines[i].strip()

        # Check if line is a date (format: "November 30, 2025")
        if re.match(r'^[A-Za-z]+ \d{1,2}, \d{4}$', line):
            current_date = line
            i += 1
            continue

        # Check if line is a time (format: "01:08 PM")
        if re.match(r'^\d{1,2}:\d{2} [AP]M$', line):
            current_time = line
            i += 1
            continue

        # Collect text until next date or time marker
        if current_date and current_time and line:
            # This is the transcription text
            logs_by_date[current_date].append({
                'time': current_time,
                'text': line
            })

        i += 1

    return logs_by_date

def count_words(text):
    """Count words in text, ignoring empty lines and special entries."""
    if not text or text.lower() in ['audio is silent.', 'the transcription was dismissed.']:
        return 0
    return len(text.split())

def analyze_logs(logs_by_date):
    """
    Analyze logs and calculate statistics.

    Returns:
        dict: Statistics including words per day, total words, etc.
    """
    daily_stats = {}
    total_words = 0
    days_with_recordings = 0

    for date, recordings in sorted(logs_by_date.items()):
        words_count = 0
        recording_count = 0
        silent_count = 0

        for record in recordings:
            words = count_words(record['text'])
            if words > 0:
                words_count += words
                recording_count += 1
            else:
                silent_count += 1

        if recording_count > 0:
            days_with_recordings += 1

        total_words += words_count
        daily_stats[date] = {
            'words': words_count,
            'recordings': recording_count,
            'silent': silent_count,
            'total_entries': len(recordings)
        }

    return {
        'daily_stats': daily_stats,
        'total_words': total_words,
        'total_days': len(daily_stats),
        'days_with_recordings': days_with_recordings,
    }

def estimate_api_costs(total_words, total_days):
    """
    Estimate OpenAI API costs based on word count.

    OpenAI pricing (as of Dec 2025):
    - Whisper API: ~$0.02 per minute of audio
    - Average speech rate: ~150 words per minute

    Returns:
        dict: Cost estimates
    """

    # Estimate audio duration from word count
    # Average speaking rate: 150 words per minute
    avg_words_per_minute = 150
    estimated_minutes = total_words / avg_words_per_minute

    # Whisper API costs
    whisper_cost_per_minute = 0.02  # USD
    whisper_total_cost = estimated_minutes * whisper_cost_per_minute

    # Monthly estimate (assuming current usage pattern continues)
    if total_days > 0:
        avg_words_per_day = total_words / total_days
        monthly_words = avg_words_per_day * 30
        monthly_minutes = monthly_words / avg_words_per_minute
        monthly_cost = monthly_minutes * whisper_cost_per_minute
    else:
        monthly_cost = 0

    return {
        'total_words': total_words,
        'estimated_minutes': round(estimated_minutes, 2),
        'total_whisper_cost': round(whisper_total_cost, 4),
        'monthly_estimated_words': round(monthly_words, 0) if total_days > 0 else 0,
        'monthly_estimated_minutes': round(monthly_minutes, 2) if total_days > 0 else 0,
        'monthly_estimated_cost': round(monthly_cost, 2) if total_days > 0 else 0,
    }

def group_by_month(logs_by_date):
    """
    Group daily stats by month.

    Returns:
        dict: {month_year: {'words': int, 'days': int, 'minutes': float, 'cost': float}}
    """
    monthly_stats = defaultdict(lambda: {'words': 0, 'days': 0})

    for date_str in logs_by_date.keys():
        # Parse date like "November 30, 2025"
        try:
            date_obj = datetime.strptime(date_str, "%B %d, %Y")
            month_year = date_obj.strftime("%B %Y")  # "November 2025"
        except ValueError:
            continue

        # Count words for this day
        day_words = 0
        for record in logs_by_date[date_str]:
            day_words += count_words(record['text'])

        if day_words > 0:
            monthly_stats[month_year]['words'] += day_words
            monthly_stats[month_year]['days'] += 1

    # Calculate minutes and costs
    for month_year in monthly_stats:
        words = monthly_stats[month_year]['words']
        monthly_stats[month_year]['minutes'] = round(words / 150, 2)
        monthly_stats[month_year]['cost'] = round(monthly_stats[month_year]['minutes'] * 0.02, 2)

    return monthly_stats

def main():
    filepath = '/Users/vitaliizinchenko/Projects/typefree/notes/wf-log.txt'

    print("Parsing Whisper workflow log...\n")
    logs_by_date = parse_log_file(filepath)

    print("Analyzing logs...\n")
    stats = analyze_logs(logs_by_date)

    # Print daily breakdown
    print("=" * 80)
    print("DAILY BREAKDOWN")
    print("=" * 80)
    for date, day_stats in stats['daily_stats'].items():
        if day_stats['words'] > 0:
            print(f"{date:20} | {day_stats['words']:5} words | {day_stats['recordings']} recordings | {day_stats['silent']} silent")

    print("\n" + "=" * 80)
    print("SUMMARY STATISTICS")
    print("=" * 80)
    print(f"Total days with entries:      {stats['total_days']}")
    print(f"Days with actual recordings:  {stats['days_with_recordings']}")
    print(f"Total words transcribed:      {stats['total_words']:,}")

    # Calculate costs
    costs = estimate_api_costs(stats['total_words'], stats['days_with_recordings'])

    print("\n" + "=" * 80)
    print("OPENAI API COST ESTIMATE (Whisper)")
    print("=" * 80)
    print(f"Estimated audio duration:     {costs['estimated_minutes']:,.2f} minutes")
    print(f"Total cost (all recordings):  ${costs['total_whisper_cost']:.4f}")
    print()
    print("MONTHLY PROJECTION (if pattern continues):")
    print(f"Avg words per day:            {stats['total_words'] / max(1, stats['days_with_recordings']):,.0f}")
    print(f"Estimated monthly words:      {costs['monthly_estimated_words']:,.0f}")
    print(f"Estimated monthly minutes:    {costs['monthly_estimated_minutes']:,.2f}")
    print(f"Estimated monthly cost:       ${costs['monthly_estimated_cost']:.2f}")
    print("=" * 80)

    # Monthly breakdown
    print("\n" + "=" * 80)
    print("MONTHLY BREAKDOWN")
    print("=" * 80)
    monthly_stats = group_by_month(logs_by_date)

    # Sort by date
    sorted_months = sorted(monthly_stats.keys(),
                          key=lambda x: datetime.strptime(x, "%B %Y"))

    print(f"{'Month':<20} | {'Words':>8} | {'Days':>5} | {'Minutes':>8} | {'Cost':>8}")
    print("-" * 70)
    for month_year in sorted_months:
        data = monthly_stats[month_year]
        print(f"{month_year:<20} | {data['words']:>8,} | {data['days']:>5} | {data['minutes']:>8.2f} | ${data['cost']:>7.2f}")
    print("=" * 80)

if __name__ == '__main__':
    main()
