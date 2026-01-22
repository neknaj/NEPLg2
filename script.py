import os

def merge_directory_files(src_directory, output_filename):
    """
    指定されたディレクトリ内の全てのファイルを探索し、一つのテキストファイルにまとめます。

    Args:
        src_directory (str): 探索するソースディレクトリのパス
        output_filename (str): 出力するファイルのパス
    """
    # srcディレクトリが存在するか確認
    if not os.path.isdir(src_directory):
        print(f"エラー: ディレクトリ '{src_directory}' が見つかりません。スキップします。")
        return

    # 出力ファイルのディレクトリが存在しない場合は作成する
    output_dir = os.path.dirname(output_filename)
    if output_dir and not os.path.exists(output_dir):
        os.makedirs(output_dir)
        print(f"出力先ディレクトリを作成しました: {output_dir}")

    print(f"--- 開始: '{src_directory}' から '{output_filename}' への結合 ---")

    try:
        # 出力ファイルを開く (UTF-8で書き込み)
        with open(output_filename, 'w', encoding='utf-8') as outfile:
            file_count = 0
            # os.walkを使ってディレクトリを再帰的に探索
            for dirpath, dirnames, filenames in os.walk(src_directory):
                # ファイル名順不同にならないようソートして処理
                for filename in sorted(filenames):
                    filepath = os.path.join(dirpath, filename)

                    # 出力ファイル自身を読み込まないようにする（同じディレクトリに出力する場合など）
                    if os.path.abspath(filepath) == os.path.abspath(output_filename):
                        continue

                    print(f"処理中: {filepath}")
                    file_count += 1

                    # ファイルパスを書き込む（実行場所からの相対パスで見やすくする）
                    relative_path = os.path.relpath(filepath, start='.')
                    outfile.write(f"{relative_path}\n---\n")

                    # ファイルの内容を読み込んで書き込む
                    try:
                        with open(filepath, 'r', encoding='utf-8') as infile:
                            content = infile.read()
                            outfile.write(content)
                    except UnicodeDecodeError:
                         outfile.write(f"\n--- エラー: ファイル '{relative_path}' はUTF-8でデコードできませんでした（バイナリファイルの可能性があります） ---\n")
                    except Exception as e:
                        outfile.write(f"\n--- エラー: ファイル '{relative_path}' を読み込めませんでした: {e} ---\n")

                    # 内容と次のファイルの間に区切り線を入れる
                    outfile.write("\n---\n\n")

        print(f"完了: {file_count} 個のファイルを '{output_filename}' にまとめました。\n")

    except Exception as e:
        print(f"予期せぬエラーが発生しました: {e}")


if __name__ == '__main__':
    # 例1: ./src ディレクトリを src.txt にまとめる
    merge_directory_files('./nepl-cli', './tmp/cli.txt')
    merge_directory_files('./nepl-core', './tmp/core.txt')
    merge_directory_files('./stdlib', './tmp/stdlib.txt')
    merge_directory_files('./examples', './tmp/examples.txt')
    merge_directory_files('./src', './tmp/src.txt')